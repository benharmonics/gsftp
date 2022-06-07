use tui::widgets::ListState;
use ssh2::Session;

use crate::readdir::{DirBuf, DirContents};

#[derive(Debug)]
pub enum ActiveState {
    Local,
    Remote,
}

#[derive(Debug)]
pub struct AppState {
    pub local: ListState,
    pub remote: ListState,
    pub active: ActiveState,
}

impl AppState {
    pub fn new() -> AppState {
        let mut local = ListState::default();
        let mut remote = ListState::default();
        local.select(Some(0));
        remote.select(Some(0));
        let active = ActiveState::Local;

        AppState { local, remote, active, }
    }
}

#[derive(Debug)]
pub struct App {
    pub buf: DirBuf,
    pub content: DirContents,
    pub state: AppState,
    pub show_help: bool,
}

impl App {
    pub fn from(buf: DirBuf, sess: &Session) -> App {
        let content = DirContents::from(&buf, sess);
        let state = AppState::new();
        let show_help = false;

        App { buf, content, state, show_help }
    }

    pub fn cd_into_local(&mut self) {
        let i = self.state.local.selected().unwrap_or(0);
        self.buf.local.push(&self.content.local[i]);
        if !self.buf.local.is_dir() { self.buf.local.pop(); return }
        self.content.update_local(&self.buf.local);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }
    
    pub fn cd_out_of_local(&mut self) {
        self.buf.local.pop();
        self.content.update_local(&self.buf.local);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }
}