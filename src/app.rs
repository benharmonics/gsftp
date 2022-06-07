use tui::widgets::ListState;
use ssh2::Session;

use crate::config::Config;
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
    pub show_hidden: bool,
}

impl App {
    pub fn from(buf: DirBuf, sess: &Session, conf: &Config) -> App {
        let show_hidden = conf.show_hidden;
        let content = DirContents::from(&buf, sess, show_hidden);
        let state = AppState::new();
        let show_help = false;

        App { buf, content, state, show_help, show_hidden }
    }

    pub fn cd_into_local(&mut self) {
        let i = self.state.local.selected().unwrap_or(0);
        self.buf.local.push(&self.content.local[i]);
        if !self.buf.local.is_dir() { self.buf.local.pop(); return }
        self.content.update_local(&self.buf.local, self.show_hidden);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }
    
    pub fn cd_out_of_local(&mut self) {
        self.buf.local.pop();
        self.content.update_local(&self.buf.local, self.show_hidden);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }

    pub fn cd_into_remote(&mut self, sess: &Session) {
        let i = self.state.remote.selected().unwrap_or(0);
        self.buf.remote.push(&self.content.remote[i]);
        self.content.update_remote(&sess, &self.buf.remote, self.show_hidden);
        // Can't use .is_dir() method on the remote connection, so we have to do this janky check -
        // making sure we don't treat files as if they're directories
        if self.content.remote.first().unwrap_or(&String::new()) == self.buf.remote.as_os_str().to_str().unwrap_or_default()
        {
            self.buf.remote.pop();
            self.content.update_remote(&sess, &self.buf.remote, self.show_hidden);
            return
        }
        self.state.remote = ListState::default();
        self.state.remote.select(Some(0));
    }

    pub fn cd_out_of_remote(&mut self, sess: &Session) {
        self.buf.remote.pop();
        self.content.update_remote(&sess, &self.buf.remote, self.show_hidden);
        self.state.remote = ListState::default();
        self.state.remote.select(Some(0));
    }
}