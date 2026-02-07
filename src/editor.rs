use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;
use tui_textarea::TextArea;

pub fn open(title: &str) -> Result<Option<String>> {
    terminal::enable_raw_mode().context("failed to enable raw mode")?;
    io::stdout()
        .execute(EnterAlternateScreen)
        .context("failed to enter alternate screen")?;

    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    let result = run_editor(&mut terminal, title);

    terminal::disable_raw_mode().ok();
    io::stdout().execute(LeaveAlternateScreen).ok();

    result
}

fn run_editor(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    title: &str,
) -> Result<Option<String>> {
    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(" {title} ")),
    );
    textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    textarea.set_line_number_style(Style::default().fg(Color::DarkGray));

    loop {
        terminal.draw(|frame| {
            let chunks =
                Layout::vertical([Constraint::Min(5), Constraint::Length(2)]).split(frame.area());

            frame.render_widget(&textarea, chunks[0]);

            let help = Paragraph::new(vec![Line::from(vec![
                Span::styled(
                    " Ctrl+D ",
                    Style::default().fg(Color::Black).bg(Color::Cyan),
                ),
                Span::raw(" Submit  "),
                Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Red)),
                Span::raw(" Cancel  "),
                Span::styled(
                    " Enter ",
                    Style::default().fg(Color::Black).bg(Color::DarkGray),
                ),
                Span::raw(" New line"),
            ])]);
            frame.render_widget(help, chunks[1]);
        })?;

        if let Event::Key(key) = event::read().context("failed to read input event")? {
            match key {
                // Ctrl+D → submit
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    let text = textarea.lines().join("\n").trim().to_string();
                    if text.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(text));
                }
                // Esc → cancel
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    return Ok(None);
                }
                // Everything else → forward to textarea
                input => {
                    textarea.input(input);
                }
            }
        }
    }
}
