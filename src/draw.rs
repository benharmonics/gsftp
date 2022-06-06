use tui::{
    backend::Backend, 
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    layout::{Layout, Constraint, Direction, Rect},
    Frame, Terminal, style::{Style, Color},
};
use ssh2::Session;

use crate::config::Config;
use crate::readdir::{DirBuf, DirContents};

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, dirs: &DirBuf, sess: &Session, conf: &Config) {
    terminal.draw(|f| {
        if conf.fullscreen {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());
            ui(f, chunks[0], dirs, sess)
        } else {
            let chunks = Layout::default()
                .constraints([Constraint::Ratio(8, 10), Constraint::Ratio(2, 10)].as_ref())
                .split(f.size());
            ui(f, chunks[0], dirs, sess);
            help(f, chunks[1])
        }
    })
    .unwrap_or_else(|e| {
        eprintln!("Fatal error writing to terminal: {e}");
        std::process::exit(1);
    });
}

fn ui<B: Backend>(f: &mut Frame<B>, area: Rect, dirs: &DirBuf, sess: &Session) {
    let contents = DirContents::from(dirs, sess);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50); 2].as_ref())
        .split(area);

    let local_items: Vec<ListItem> = contents.local
        .iter()
        .map(|s| ListItem::new(s.as_ref()))
        .collect();
    let local_block = List::new(local_items)
        .block(Block::default().title(dirs.local.to_str().unwrap_or("Local")).borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Cyan))
        .highlight_symbol(">>");
    f.render_widget(local_block, chunks[0]);

    let remote_items: Vec<ListItem> = contents.remote
        .iter()
        .map(|s| ListItem::new(s.as_ref()))
        .collect();
    let remote_block = List::new(remote_items)
        .block(Block::default().title(dirs.remote.to_str().unwrap_or("Remote")).borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Cyan))
        .highlight_symbol(">>");
    f.render_widget(remote_block, chunks[1]);
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