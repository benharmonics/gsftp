//! Drawing items to the terminal
use tui::{
    backend::Backend, Frame, 
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    Terminal, 
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, ActiveState};

/// Draw a windowed terminal for our contents - the left window for our local connection,
/// and the right window for our remote connection.
/// Also draw a help menu (keyboard shortcuts) if the --fullscreen flag was not used.
pub fn ui<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    terminal.draw(|f| {
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

fn windows<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50); 2].as_ref())
        .split(area);
    
    let local_is_active = if let ActiveState::Local = app.state.active { true } else { false };

    let local_block = contents_block(local_is_active, &app.buf.local, &app.content.local);
    f.render_stateful_widget(local_block, chunks[0], &mut app.state.local);

    let remote_block = contents_block(!local_is_active, &app.buf.remote, &app.content.remote);
    f.render_stateful_widget(remote_block, chunks[1], &mut app.state.remote);
}

fn contents_block<'a>(
    active: bool,
    buf: &'a std::path::PathBuf,
    contents: &'a[String],
) -> List<'a> {
    let items: Vec<ListItem> = contents
        .iter()
        .map(|s| {
            ListItem::new(s.as_ref())
        })
        .collect();
    let highlight_color = if active { Color::Cyan } else { Color::Blue };

    List::new(items)
        .block(Block::default().title(buf.to_str().unwrap_or("Remote")).borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(highlight_color).add_modifier(Modifier::BOLD))
        .highlight_symbol(">>")
}

fn help<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_table = Table::new(vec![
            Row::new(vec!["Hello", "My", "Darling", "Hello"])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["And", "Some", "Other", "Stuff"])
                .style(Style::default().fg(Color::White)),
            Row::new(vec!["And", "One", "Last", "Row"])
                .style(Style::default().fg(Color::White)),
        ])
        .style(Style::default().fg(Color::LightYellow))
        .block(Block::default().title("Keyboard controls").borders(Borders::ALL))
        .widths([Constraint::Percentage(25); 4].as_ref());
    f.render_widget(help_table, area);
}

/// Message to display while we wait for the TCP handshake to complete.
pub fn startup_text<B: Backend>(terminal: &mut Terminal<B>) {
    terminal.draw(|f| {
        let paragraph = Paragraph::new("Connecting to client...")
            .style(Style::default().fg(Color::White));
        f.render_widget(paragraph, f.size());
    })
    .unwrap_or_else(|e| {
        eprintln!("Fatal error writing to terminal: {e}");
        std::process::exit(1);
    });
}
/// Just like the normal UI, but with a message in the bottom right corner.
pub fn text_alert<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, text: &str) {
    terminal.draw(|f| {
        if app.show_help {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(75), Constraint::Percentage(5), Constraint::Percentage(20)].as_ref())
                .split(f.size());
            windows(f, chunks[0], app);
            right_aligned_text(f, chunks[1], text);
            help(f, chunks[2]);
        } else {
            let chunks = Layout::default()
                .constraints([Constraint::Ratio(24, 25), Constraint::Ratio(1, 25)].as_ref())
                .split(f.size());
            windows(f, chunks[0], app);
            right_aligned_text(f, chunks[1], text);
        }
    })
    .unwrap_or_else(|e| {
        eprintln!("Fatal error writing to terminal: {e}");
        std::process::exit(1);
    });
}

fn right_aligned_text<B: Backend>(f: &mut Frame<B>, area: Rect, text: &str) {
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(tui::layout::Alignment::Right);
    f.render_widget(paragraph, area);
}
