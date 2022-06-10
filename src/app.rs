//! Mutable application state and utils
use tui::widgets::ListState;
use ssh2::Session;

use crate::app_utils::{AppBuf, AppContent};

#[derive(Debug)]
/// Whichever connection is 'active' (either the local or remote connections) will respond
/// to user input. The other will be in a quiescent state.
pub enum ActiveState {
    Local,
    Remote,
}

#[derive(Debug)]
/// Each of our connections (local and remote) have an associated `tui::widgets::ListState`
/// that keeps track of which `tui::widgets::ListItem` is currently selected.
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
/// The mutable configuration for our program.
pub struct App {
    pub buf: AppBuf,
    pub content: AppContent,
    pub state: AppState,
    pub show_help: bool,
    pub show_hidden: bool,
}

impl App {
    pub fn from(sess: &Session, args: clap::ArgMatches) -> App {
        let buf = AppBuf::from(sess);
        let state = AppState::new();
        let show_help = args.is_present("shortcuts");
        let show_hidden = args.is_present("all");
        let content = AppContent::from(&buf, sess, show_hidden);

        App { buf, content, state, show_help, show_hidden }
    }

    /// Updates the `DirBuf.local`, `DirContent.local` and `AppState.local`,
    /// using the currently selected item as a PathBuf, the contents of which will
    /// be read into `DirContent.local` while the PathBuf itself will be saved as
    /// `DirBuf.local`. `AppState.local` is reset to `Some(0)`.
    pub fn cd_into_local(&mut self) {
        let i = self.state.local.selected().unwrap_or(0);
        self.buf.local.push(&self.content.local[i]);
        if !self.buf.local.is_dir() { self.buf.local.pop(); return }
        self.content.update_local(&self.buf.local, self.show_hidden);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }
    
    /// Changes `DirBuf.local` to its parent, and reads the new `PathBuf`'s contents to
    /// `DirContent.local`.
    pub fn cd_out_of_local(&mut self) {
        self.buf.local.pop();
        self.content.update_local(&self.buf.local, self.show_hidden);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }

    /// Updates the `DirBuf.remote`, `DirContent.remote` and `AppState.remote`,
    /// using the currently selected item as a PathBuf, the contents of which will
    /// be read into `DirContent.remote` while the PathBuf itself will be saved as
    /// `DirBuf.remote`. `AppState.remote` is reset to `Some(0)`.
    pub fn cd_into_remote(&mut self, sess: &Session) {
        if self.content.remote.is_empty() { return }
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

    /// Changes `DirBuf.remote` to its parent, and reads the new `PathBuf`'s contents to
    /// `DirContent.remote`.
    pub fn cd_out_of_remote(&mut self, sess: &Session) {
        self.buf.remote.pop();
        self.content.update_remote(&sess, &self.buf.remote, self.show_hidden);
        self.state.remote = ListState::default();
        self.state.remote.select(Some(0));
    }
}