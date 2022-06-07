use tui::{
    backend::Backend, Frame, 
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    Terminal, 
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, ActiveState};
use crate::config::Config;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, conf: &Config) {
    terminal.draw(|f| {
        if conf.fullscreen && !app.show_help {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());
            ui(f, chunks[0], app)
        } else {
            let chunks = Layout::default()
                .constraints([Constraint::Ratio(8, 10), Constraint::Ratio(2, 10)].as_ref())
                .split(f.size());
            ui(f, chunks[0], app);
            help(f, chunks[1])
        }
    })
    .unwrap_or_else(|e| {
        eprintln!("Fatal error writing to terminal: {e}");
        std::process::exit(1);
    });
}

fn ui<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
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
                .style(Style::default().fg(Color::White))
        ])
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().title("Help").borders(Borders::ALL))
        .widths([Constraint::Percentage(25); 4].as_ref());
    f.render_widget(help_table, area);
}

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