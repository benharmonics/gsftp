use tui::widgets::ListState;
use ssh2::Session;

use crate::readdir::{DirBuf, DirContents};

#[derive(Debug)]
pub struct AppState {
    pub local: ListState,
    pub remote: ListState,
}

impl AppState {
    pub fn new() -> AppState {
        let mut local = ListState::default();
        let mut remote = ListState::default();
        local.select(Some(0));
        remote.select(Some(0));

        AppState { local, remote }
    }
}

#[derive(Debug)]
pub struct App {
    pub buf: DirBuf,
    pub content: DirContents,
    pub state: AppState,
}

impl App {
    pub fn from(buf: DirBuf, sess: &Session) -> App {
        let content = DirContents::from(&buf, sess);
        let state = AppState::new();

        App { buf, content, state }
    }
}