#Requires -Version 5.1

<#
.SYNOPSIS
    qwen-tts installer for Windows.

.DESCRIPTION
    This script:
      1. Checks prerequisites (Python 3.10+, git, cargo)
      2. Builds and installs the qwen-tts binary via cargo
      3. Creates the ~/.qwen-tts/ directory structure
      4. Creates a Python virtual environment at ~/.qwen-tts/venv/
      5. Installs the correct Python dependencies (with CUDA if NVIDIA GPU detected)
      6. Copies generate_compat.py to ~/.qwen-tts/
      7. Ensures ~/.cargo/bin is on the User PATH
      8. Runs `qwen-tts config init` to auto-detect platform settings

.EXAMPLE
    irm https://raw.githubusercontent.com/andreisuslov/qwen-tts/main/scripts/install.ps1 | iex
    # or
    .\scripts\install.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# =============================================================================
# Color helpers
# =============================================================================
function Write-Info {
    param([string]$Message)
    Write-Host "[info]  " -ForegroundColor Cyan -NoNewline
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "[ok]    " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[warn]  " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Error2 {
    param([string]$Message)
    Write-Host "[error] " -ForegroundColor Red -NoNewline
    Write-Host $Message
}

function Exit-WithError {
    param([string]$Message)
    Write-Error2 $Message
    exit 1
}

# =============================================================================
# Prerequisite checks
# =============================================================================
function Test-CommandExists {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

function Test-NvidiaGpu {
    <#
    .SYNOPSIS
        Returns $true if nvidia-smi is available and reports a GPU.
    #>
    try {
        $null = & nvidia-smi 2>$null
        return $LASTEXITCODE -eq 0
    } catch {
        return $false
    }
}

function Assert-Python {
    <#
    .SYNOPSIS
        Locates Python 3.10+ and sets $script:PythonCmd.
    #>
    $script:PythonCmd = $null

    foreach ($cmd in @("python", "python3")) {
        if (Test-CommandExists $cmd) {
            $script:PythonCmd = $cmd
            break
        }
    }

    if (-not $script:PythonCmd) {
        Exit-WithError @"
Python not found. Please install Python 3.10 or later.
Download from: https://www.python.org/downloads/
Make sure to check "Add Python to PATH" during installation.
"@
    }

    # Verify version
    $versionOutput = & $script:PythonCmd -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>&1
    $major = & $script:PythonCmd -c "import sys; print(sys.version_info.major)" 2>&1
    $minor = & $script:PythonCmd -c "import sys; print(sys.version_info.minor)" 2>&1

    if ([int]$major -lt 3 -or ([int]$major -eq 3 -and [int]$minor -lt 10)) {
        Exit-WithError "Python 3.10+ required, but found $versionOutput ($($script:PythonCmd)). Please upgrade."
    }

    Write-Success "Python $versionOutput found ($($script:PythonCmd))"
}

function Assert-Git {
    if (-not (Test-CommandExists "git")) {
        Exit-WithError @"
git not found. Please install git.
Download from: https://git-scm.com/download/win
"@
    }
    $gitVersion = & git --version 2>&1
    Write-Success "git found ($gitVersion)"
}

function Assert-Cargo {
    # If cargo is not on PATH, try sourcing the env
    if (-not (Test-CommandExists "cargo")) {
        $cargoEnv = Join-Path $env:USERPROFILE ".cargo\env.ps1"
        if (Test-Path $cargoEnv) {
            . $cargoEnv
        }
    }

    if (-not (Test-CommandExists "cargo")) {
        Exit-WithError @"
cargo (Rust) not found. Please install Rust first:
  Visit https://rustup.rs and download rustup-init.exe
  Or run:  winget install Rustlang.Rustup
Then restart your terminal and re-run this script.
"@
    }
    $cargoVersion = & cargo --version 2>&1
    Write-Success "cargo found ($cargoVersion)"
}

function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    Assert-Python
    Assert-Git
    Assert-Cargo
    Write-Host ""
}

# =============================================================================
# Install the Rust binary
# =============================================================================
function Install-Binary {
    Write-Info "Building and installing qwen-tts binary..."

    # Detect if running from inside the repo
    $scriptDir = Split-Path -Parent $MyInvocation.ScriptName
    # Handle the case where the script is piped (no ScriptName)
    if (-not $scriptDir) {
        $scriptDir = Get-Location
    }
    $repoRoot = Split-Path -Parent $scriptDir
    $cargoToml = Join-Path $repoRoot "Cargo.toml"

    if ((Test-Path $cargoToml) -and (Select-String -Path $cargoToml -Pattern 'name = "qwen-tts"' -Quiet)) {
        Write-Info "Local repository detected at $repoRoot - building from source"
        & cargo install --path $repoRoot
    } else {
        Write-Info "Installing from remote repository..."
        & cargo install --git https://github.com/andreisuslov/qwen-tts
    }

    if ($LASTEXITCODE -ne 0) {
        Exit-WithError "Failed to build qwen-tts binary"
    }

    $binaryPath = Join-Path $env:USERPROFILE ".cargo\bin\qwen-tts.exe"
    Write-Success "qwen-tts binary installed to $binaryPath"
}

# =============================================================================
# Create directory structure
# =============================================================================
$QwenHome = Join-Path $env:USERPROFILE ".qwen-tts"

function New-DirectoryStructure {
    Write-Info "Creating directory structure at $QwenHome ..."

    $dirs = @(
        (Join-Path $QwenHome "models"),
        (Join-Path $QwenHome "voices"),
        (Join-Path $QwenHome "outputs")
    )

    foreach ($dir in $dirs) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }

    Write-Success "Directory structure created"
}

# =============================================================================
# Python virtual environment and dependencies
# =============================================================================
function Install-PythonDeps {
    $venvDir = Join-Path $QwenHome "venv"
    $venvPython = Join-Path $venvDir "Scripts\python.exe"
    $venvPip = Join-Path $venvDir "Scripts\pip.exe"

    if (Test-Path $venvDir) {
        Write-Info "Python venv already exists at $venvDir - reusing"
    } else {
        Write-Info "Creating Python virtual environment at $venvDir ..."
        & $script:PythonCmd -m venv $venvDir
        if ($LASTEXITCODE -ne 0) {
            Exit-WithError "Failed to create Python virtual environment"
        }
        Write-Success "Python venv created"
    }

    Write-Info "Upgrading pip..."
    & $venvPython -m pip install --upgrade pip --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Warn "pip upgrade returned non-zero exit code, continuing..."
    }

    Write-Info "Installing Python dependencies..."

    $hasNvidia = Test-NvidiaGpu

    if ($hasNvidia) {
        Write-Info "NVIDIA GPU detected - installing PyTorch with CUDA support"
        & $venvPip install transformers torch torchaudio --index-url https://download.pytorch.org/whl/cu121 --quiet
        if ($LASTEXITCODE -ne 0) {
            Write-Warn "CUDA torch install failed, falling back to CPU torch..."
            & $venvPip install transformers torch torchaudio --quiet
        }
        & $venvPip install huggingface_hub soundfile --quiet
        Write-Success "Installed transformers, torch (CUDA), torchaudio, huggingface_hub, soundfile"
    } else {
        Write-Info "No NVIDIA GPU detected - installing CPU-only PyTorch"
        & $venvPip install transformers torch torchaudio huggingface_hub soundfile --quiet
        if ($LASTEXITCODE -ne 0) {
            Exit-WithError "Failed to install Python dependencies"
        }
        Write-Success "Installed transformers, torch, torchaudio, huggingface_hub, soundfile"
    }
}

# =============================================================================
# Copy generate_compat.py
# =============================================================================
function Copy-GenerateCompat {
    $scriptDir = Split-Path -Parent $MyInvocation.ScriptName
    if (-not $scriptDir) {
        $scriptDir = Get-Location
    }
    $repoRoot = Split-Path -Parent $scriptDir
    $dest = Join-Path $QwenHome "generate_compat.py"

    # Look for generate_compat.py in several likely locations
    $candidates = @(
        (Join-Path $repoRoot "generate_compat.py"),
        (Join-Path $repoRoot "scripts\generate_compat.py"),
        (Join-Path $scriptDir "generate_compat.py")
    )

    $src = $null
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            $src = $candidate
            break
        }
    }

    if ($src) {
        Copy-Item -Path $src -Destination $dest -Force
        Write-Success "Copied generate_compat.py to $dest"
    } elseif (Test-Path $dest) {
        Write-Info "generate_compat.py already present at $dest"
    } else {
        Write-Warn "generate_compat.py not found in repository ($src)."
        Write-Warn "You may need to download it manually or run: qwen-tts models download"
        Write-Warn "The file should be placed at $dest"
    }
}

# =============================================================================
# Ensure ~/.cargo/bin is on User PATH
# =============================================================================
function Set-CargoPath {
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"

    # Check if already on the User PATH
    $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")

    if ($currentUserPath -and ($currentUserPath -split ";" | ForEach-Object { $_.TrimEnd("\") }) -contains $cargoBin.TrimEnd("\")) {
        Write-Info "$cargoBin is already on User PATH"
        return
    }

    Write-Info "Adding $cargoBin to User PATH..."

    if ($currentUserPath) {
        # Avoid duplicate trailing semicolons
        $newPath = ($currentUserPath.TrimEnd(";") + ";$cargoBin")
    } else {
        $newPath = $cargoBin
    }

    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Success "User PATH updated (restart your terminal for changes to take effect)"

    # Also update the current session so the rest of this script can find the binary
    if ($env:Path -notlike "*$cargoBin*") {
        $env:Path = "$cargoBin;$env:Path"
    }
}

# =============================================================================
# Run qwen-tts config init
# =============================================================================
function Invoke-ConfigInit {
    Write-Info "Initializing qwen-tts configuration..."

    $binary = Join-Path $env:USERPROFILE ".cargo\bin\qwen-tts.exe"

    if (Test-Path $binary) {
        & $binary config init
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Configuration initialized"
        } else {
            Write-Warn "qwen-tts config init returned non-zero exit code"
        }
    } elseif (Test-CommandExists "qwen-tts") {
        & qwen-tts config init
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Configuration initialized"
        } else {
            Write-Warn "qwen-tts config init returned non-zero exit code"
        }
    } else {
        Write-Warn "qwen-tts binary not found - skipping config init"
        Write-Warn "Run 'qwen-tts config init' manually after restarting your terminal"
    }
}

# =============================================================================
# Print summary
# =============================================================================
function Write-Summary {
    Write-Host ""
    Write-Host "============================================" -ForegroundColor Green
    Write-Host "  qwen-tts installed successfully!" -ForegroundColor Green
    Write-Host "============================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Binary:    $env:USERPROFILE\.cargo\bin\qwen-tts.exe"
    Write-Host "  Data dir:  $env:USERPROFILE\.qwen-tts\"
    Write-Host "  Venv:      $env:USERPROFILE\.qwen-tts\venv\"

    # Config path depends on the platform's config dir
    $configDir = [Environment]::GetFolderPath("ApplicationData")
    Write-Host "  Config:    $configDir\qwen-tts\config.toml"

    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  1. Restart your terminal (PATH has been updated)"
    Write-Host "  2. Download a model:"
    Write-Host "       qwen-tts models download --variant pro"
    Write-Host "  3. Generate speech:"
    Write-Host "       qwen-tts speak `"Hello, world!`""
    Write-Host ""

    if (Test-NvidiaGpu) {
        Write-Host "  Backend: CUDA (NVIDIA GPU detected)" -ForegroundColor Green
    } else {
        Write-Host "  Backend: CPU" -ForegroundColor Yellow
    }
    Write-Host ""
}

# =============================================================================
# Main
# =============================================================================
function Main {
    Write-Host ""
    Write-Host "qwen-tts installer" -ForegroundColor White
    Write-Host "==================" -ForegroundColor White
    Write-Host ""

    Test-Prerequisites

    Install-Binary
    Write-Host ""

    New-DirectoryStructure
    Write-Host ""

    Install-PythonDeps
    Write-Host ""

    Copy-GenerateCompat
    Write-Host ""

    Set-CargoPath
    Write-Host ""

    Invoke-ConfigInit
    Write-Host ""

    Write-Summary
}

# Run the installer
Main
