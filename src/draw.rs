//! Drawing items to the terminal
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};

use crate::app::App;
use crate::app_utils::ActiveState;

/// Contains information about window text, allows for drawing to the terminal
pub struct UiWindow {
    text: Option<String>,
    style: Option<TextStyle>,
}

impl UiWindow {
    pub fn default() -> UiWindow {
        let text = Some(String::from("Press '?' to toggle help"));
        let style = Some(TextStyle::default());
        UiWindow { text, style }
    }

    pub fn reset(&mut self) {
        self.text = None;
        self.style = None;
    }

    pub fn flashing_text(&mut self, text: &str) {
        self.text = Some(String::from(text));
        self.style = Some(TextStyle::flash());
    }

    pub fn error_message(&mut self, text: &str) {
        self.text = Some(String::from(text));
        self.style = Some(TextStyle::error());
    }

    pub fn text(&self) -> Option<&String> {
        self.text.as_ref()
    }

    pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>, app: &mut App) {
        // TODO: remove clone
        match self.text {
            Some(_) => {
                let window = self.clone();
                text_alert(terminal, app, window)
            }
            None => basic_ui(terminal, app),
        }
    }
}

impl Clone for UiWindow {
    fn clone(&self) -> UiWindow {
        let text = match &self.text {
            Some(t) => Some(String::from(t)),
            None => None,
        };
        let style = match &self.style {
            Some(s) => Some(TextStyle::from(s)),
            None => None,
        };

        UiWindow { text, style }
    }
}

// This struct here reduces code repetition in main.rs and also prevents text styling
// from being overlooked/changed from the default implementations.
// Provides default implementations for text styling
struct TextStyle {
    color: Color,
    modifier: Option<Modifier>,
}

impl TextStyle {
    fn from(text_style: &TextStyle) -> TextStyle {
        let color = text_style.color.clone();
        let modifier = match &text_style.modifier {
            Some(m) => Some(m.clone()),
            None => None,
        };

        TextStyle { color, modifier }
    }

    fn default() -> TextStyle {
        TextStyle {
            color: Color::LightCyan,
            modifier: None,
        }
    }

    fn flash() -> TextStyle {
        TextStyle {
            color: Color::Cyan,
            modifier: Some(Modifier::SLOW_BLINK | Modifier::ITALIC),
        }
    }

    fn error() -> TextStyle {
        TextStyle {
            color: Color::Red,
            modifier: Some(Modifier::BOLD | Modifier::ITALIC),
        }
    }
}

// Draw a windowed terminal for our contents - the left window for our local connection,
// and the right window for our remote connection.
// Also draw a help menu (keyboard shortcuts) if the --shortcuts flag was used.
fn basic_ui<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    terminal
        .draw(|f| {
            if app.show_help {
                let chunks = Layout::default()
                    .constraints([Constraint::Ratio(4, 5), Constraint::Ratio(1, 5)].as_ref())
                    .split(f.size());
                windows(f, chunks[0], app);
                help(f, chunks[1]);
            } else {
                let chunks = Layout::default()
                    .constraints([Constraint::Ratio(24, 25), Constraint::Ratio(1, 25)].as_ref())
                    .split(f.size());
                windows(f, chunks[0], app);
            }
        })
        .unwrap_or_else(|e| {
            eprintln!("Fatal error writing to terminal: {e}");
            std::process::exit(1);
        });
}

// Divides an area into two windows & renders them using a helper function `contents_block`
fn windows<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50); 2].as_ref())
        .split(area);

    let local_is_active = matches!(app.state.active, ActiveState::Local);
    let local_block = contents_block(local_is_active, &app.buf.local, &app.content.local);
    f.render_stateful_widget(local_block, chunks[0], &mut app.state.local);

    let remote_block = contents_block(!local_is_active, &app.buf.remote, &app.content.remote);
    f.render_stateful_widget(remote_block, chunks[1], &mut app.state.remote);
}

// Draws the contents of each window
fn contents_block<'a>(active: bool, buf: &'a std::path::Path, contents: &'a [String]) -> List<'a> {
    let items: Vec<ListItem> = contents.iter().map(|s| ListItem::new(s.as_ref())).collect();
    let highlight_color = if active { Color::Cyan } else { Color::Blue };

    List::new(items)
        .block(
            Block::default()
                .title(buf.to_str().unwrap_or("Remote"))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(highlight_color)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">>")
}

// A help text window which appears at the bottom of the screen when you press '?'
fn help<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_table = Table::new(vec![
        Row::new(vec![
            "k or ⬆: move up",
            "j or ⬇: move down",
            "q or Esc: exit",
        ])
        .style(Style::default().fg(Color::White)),
        Row::new(vec![
            "l or ➡: enter directory",
            "h or ⬅: exit directory",
            "g or Ctrl+⬆: page up",
        ])
        .style(Style::default().fg(Color::White)),
        Row::new(vec![
            "y or ↩: download/upload",
            "w or ↹: switch windows",
            "b or Ctrl+⬇: page down",
        ])
        .style(Style::default().fg(Color::White)),
        Row::new(vec!["a: toggle hidden files", "?: toggle help"])
            .style(Style::default().fg(Color::White)),
    ])
    .style(Style::default().fg(Color::LightYellow))
    .block(
        Block::default()
            .title("Keyboard controls")
            .borders(Borders::ALL),
    )
    .widths([Constraint::Ratio(1, 3); 4].as_ref());
    f.render_widget(help_table, area);
}

// Just like the normal UI, but with a message in the bottom right corner.
fn text_alert<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, window: UiWindow) {
    terminal
        .draw(|f| {
            if app.show_help {
                let chunks = Layout::default()
                    .constraints(
                        [
                            Constraint::Percentage(75),
                            Constraint::Percentage(5),
                            Constraint::Percentage(20),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());
                windows(f, chunks[0], app);
                let style = window.style.unwrap_or_else(TextStyle::default);
                let text = window.text.unwrap_or(String::from("missing text"));
                right_aligned_text(f, chunks[1], text.as_str(), style);
                help(f, chunks[2]);
            } else {
                let chunks = Layout::default()
                    .constraints([Constraint::Ratio(24, 25), Constraint::Ratio(1, 25)].as_ref())
                    .split(f.size());
                windows(f, chunks[0], app);
                let style = window.style.unwrap();
                let text = window.text.unwrap_or(String::from("missing text"));
                right_aligned_text(f, chunks[1], text.as_str(), style);
            }
        })
        .unwrap_or_else(|e| {
            eprintln!("Fatal error writing to terminal: {e}");
            std::process::exit(1);
        });
}

fn right_aligned_text<B: Backend>(f: &mut Frame<B>, area: Rect, text: &str, style: TextStyle) {
    let paragraph = if let Some(modifier) = style.modifier {
        Paragraph::new(text)
            .style(Style::default().fg(style.color).add_modifier(modifier))
            .alignment(tui::layout::Alignment::Right)
    } else {
        Paragraph::new(text)
            .style(Style::default().fg(style.color))
            .alignment(tui::layout::Alignment::Right)
    };
    f.render_widget(paragraph, area)
}
