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

impl Default for UiWindow {
  fn default() -> Self {
    let text = Some(String::from("Press '?' to toggle help"));
    let style = Some(TextStyle::default());
    Self { text, style }
  }
}

impl UiWindow {
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

  /// Draw UI
  pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>, app: &mut App) {
    match self.text {
      Some(_) => text_alert(terminal, app, self),
      None => basic_ui(terminal, app),
    }
  }
}

// This struct here reduces code repetition in main.rs and also prevents text styling
// from being overlooked/changed from the default implementations.
// Provides default implementations for text styling
struct TextStyle {
  color: Color,
  modifier: Option<Modifier>,
}

impl Default for TextStyle {
  fn default() -> TextStyle {
    TextStyle {
      color: Color::LightCyan,
      modifier: None,
    }
  }
}

impl TextStyle {
  fn from(text_style: &TextStyle) -> TextStyle {
    let color = text_style.color;
    let modifier = text_style.modifier.as_ref().copied();

    TextStyle { color, modifier }
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
          .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)].as_ref())
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
      "l or ➡: enter directory",
      "g or Ctrl+⬆: page up",
    ])
    .style(Style::default().fg(Color::White)),
    Row::new(vec![
      "j or ⬇: move down",
      "h or ⬅: exit directory",
      "G or Ctrl+⬇: page down",
    ])
    .style(Style::default().fg(Color::White)),
    Row::new(vec![
      "y or ↩: download/upload",
      "w or ↹: switch windows",
      "a: toggle hidden files",
    ])
    .style(Style::default().fg(Color::White)),
    Row::new(vec!["", "q or Esc: exit", "?: toggle help"]).style(Style::default().fg(Color::White)),
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
fn text_alert<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, window: &UiWindow) {
  terminal
    .draw(|f| {
      let style = window
        .style
        .as_ref()
        .map(TextStyle::from)
        .unwrap_or_default();
      let text = window.text.as_deref().unwrap_or("[missing text]");
      if app.show_help {
        let chunks = Layout::default()
          .constraints(
            [
              Constraint::Percentage(70),
              Constraint::Percentage(5),
              Constraint::Percentage(25),
            ]
            .as_ref(),
          )
          .split(f.size());
        windows(f, chunks[0], app);
        right_aligned_text(f, chunks[1], text, style);
        help(f, chunks[2]);
      } else {
        let chunks = Layout::default()
          .constraints([Constraint::Ratio(24, 25), Constraint::Ratio(1, 25)].as_ref())
          .split(f.size());
        windows(f, chunks[0], app);
        right_aligned_text(f, chunks[1], text, style);
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
