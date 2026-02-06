use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod editor;
mod generate;
mod models;
mod output;
mod platform;
mod voices;

#[derive(Parser)]
#[command(name = "qwen-tts")]
#[command(about = "Cross-platform CLI for Qwen3-TTS text-to-speech with voice cloning")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate speech from text
    Speak {
        /// Text to speak (positional)
        text: Option<String>,

        /// Read text from a file
        #[arg(long)]
        file: Option<String>,

        /// Voice name
        #[arg(long)]
        voice: Option<String>,

        /// Emotion/style instruction (e.g. "Excited", "Calm")
        #[arg(long)]
        emotion: Option<String>,

        /// Speech speed multiplier
        #[arg(long)]
        speed: Option<f32>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Design a voice from a text description
    Design {
        /// Voice description (e.g. "A deep calm British narrator")
        description: String,

        /// Text to speak
        #[arg(long)]
        text: Option<String>,

        /// Read text from a file
        #[arg(long)]
        file: Option<String>,

        /// Speech speed multiplier
        #[arg(long)]
        speed: Option<f32>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Clone a voice from reference audio
    Clone {
        /// Path to reference audio file
        #[arg(long = "ref")]
        ref_audio: Option<String>,

        /// Transcript of the reference audio
        #[arg(long)]
        ref_text: Option<String>,

        /// Use a saved voice by name
        #[arg(long)]
        voice: Option<String>,

        /// Text to speak
        #[arg(long)]
        text: Option<String>,

        /// Read text from a file
        #[arg(long)]
        file: Option<String>,

        /// Speech speed multiplier
        #[arg(long)]
        speed: Option<f32>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Manage saved voices
    Voices {
        #[command(subcommand)]
        action: VoicesAction,
    },

    /// Manage TTS models
    Models {
        #[command(subcommand)]
        action: ModelsAction,
    },

    /// View and modify configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum VoicesAction {
    /// List all saved voices
    List,

    /// Enroll a new voice from reference audio
    Add {
        /// Name for the voice
        name: String,

        /// Path to reference audio file (.wav)
        #[arg(long = "ref")]
        ref_audio: String,

        /// Transcript of the reference audio
        #[arg(long)]
        transcript: Option<String>,
    },

    /// Remove a saved voice
    Remove {
        /// Name of the voice to remove
        name: String,
    },
}

#[derive(Subcommand)]
enum ModelsAction {
    /// List installed models
    List,

    /// Download a model
    Download {
        /// Model variant: "pro" or "lite"
        #[arg(long, default_value = "pro")]
        variant: String,
    },

    /// Update model to the latest release
    Update {
        /// Model variant to update (defaults to configured variant)
        #[arg(long)]
        variant: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Config key
        key: String,
        /// New value
        value: String,
    },

    /// Initialize configuration (auto-detect platform)
    Init,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Speak {
            text,
            file,
            voice,
            emotion,
            speed,
            output,
        } => generate::speak(generate::SpeakArgs {
            text,
            file,
            voice,
            emotion,
            speed,
            output,
        }),

        Commands::Design {
            description,
            text,
            file,
            speed,
            output,
        } => generate::design(generate::DesignArgs {
            description,
            text,
            file,
            speed,
            output,
        }),

        Commands::Clone {
            ref_audio,
            ref_text,
            voice,
            text,
            file,
            speed,
            output,
        } => generate::clone(generate::CloneArgs {
            ref_audio,
            ref_text,
            voice,
            text,
            file,
            speed,
            output,
        }),

        Commands::Voices { action } => match action {
            VoicesAction::List => voices::list(),
            VoicesAction::Add {
                name,
                ref_audio,
                transcript,
            } => voices::add(&name, &ref_audio, transcript.as_deref()),
            VoicesAction::Remove { name } => voices::remove(&name),
        },

        Commands::Models { action } => match action {
            ModelsAction::List => models::list(),
            ModelsAction::Download { variant } => models::download(&variant),
            ModelsAction::Update { variant } => models::update(variant.as_deref()),
        },

        Commands::Config { action } => match action {
            ConfigAction::Show => config::show(),
            ConfigAction::Set { key, value } => config::set(&key, &value),
            ConfigAction::Init => config::init(),
        },
    }
}
