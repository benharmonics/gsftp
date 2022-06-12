//! Mutable application state and utils
use ssh2::Session;
use tui::widgets::ListState;

use crate::app_utils::{AppBuf, AppContent, AppState};

#[derive(Debug)]
/// Static, mutable application configuration
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

    /// Updates the `AppBuf.local`, `AppContent.local` and `AppState.local`,
    /// using the currently selected item as a PathBuf, the contents of which will
    /// be read into `AppContent.local` while the PathBuf itself will be saved as
    /// `AppBuf.local`. `AppState.local` is reset to `Some(0)`.
    pub fn cd_into_local(&mut self) {
        let i = self.state.local.selected().unwrap_or(0);
        self.buf.local.push(&self.content.local[i]);
        if !self.buf.local.is_dir() { self.buf.local.pop(); return }
        self.content.update_local(&self.buf.local, self.show_hidden);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }
    
    /// Changes `AppBuf.local` to its parent, and reads the new `PathBuf`'s contents to
    /// `AppContent.local`.
    pub fn cd_out_of_local(&mut self) {
        self.buf.local.pop();
        self.content.update_local(&self.buf.local, self.show_hidden);
        self.state.local = ListState::default();
        self.state.local.select(Some(0));
    }

    /// Updates the `AppBuf.remote`, `AppContent.remote` and `AppState.remote`,
    /// using the currently selected item as a PathBuf, the contents of which will
    /// be read into `AppContent.remote` while the PathBuf itself will be saved as
    /// `AppBuf.remote`. `AppState.remote` is reset to `Some(0)`.
    pub fn cd_into_remote(&mut self, sess: &Session) {
        if self.content.remote.is_empty() { return }    // return if dir is empty, or push below will panic
        let i = self.state.remote.selected().unwrap();
        self.buf.remote.push(&self.content.remote[i]);
        // we have to make sure we don't treat files as if they're directories
        if sess.sftp().unwrap().opendir(self.buf.remote.as_path()).is_err() {
            self.buf.remote.pop();
            return
        }
        self.content.update_remote(&sess, &self.buf.remote, self.show_hidden);
        self.state.remote = ListState::default();
        self.state.remote.select(Some(0));
    }

    /// Changes `AppBuf.remote` to its parent, and reads the new `PathBuf`'s contents to
    /// `AppContent.remote`.
    pub fn cd_out_of_remote(&mut self, sess: &Session) {
        self.buf.remote.pop();
        self.content.update_remote(&sess, &self.buf.remote, self.show_hidden);
        self.state.remote = ListState::default();
        self.state.remote.select(Some(0));
    }
}