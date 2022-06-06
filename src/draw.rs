use tui::{
    backend::Backend, 
    widgets::{Block, Borders, List, ListItem},
    layout::{Layout, Constraint, Direction, Rect},
    Frame, Terminal, style::{Style, Color},
};

use crate::config::Config;
use crate::readdir::{DirBuf, DirContents};

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, dirs: &DirBuf, conf: &Config) {
    terminal.draw(|f| {
        if conf.fullscreen {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());
            ui(f, chunks[0], dirs)
        } else {
            let chunks = Layout::default()
                .constraints([Constraint::Ratio(8, 10), Constraint::Ratio(2, 10)].as_ref())
                .split(f.size());
            ui(f, chunks[0], dirs)
        }
    }).unwrap_or_else(|e| {
        eprintln!("Fatal error drawing terminal: {e}");
        std::process::exit(1);
    });
}

fn ui<B: Backend>(f: &mut Frame<B>, area: Rect, dirs: &DirBuf) {
    let contents = DirContents::from(dirs);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
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