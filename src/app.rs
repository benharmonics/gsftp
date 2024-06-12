//! Mutable application state and utils
use ssh2::{Session, Sftp};

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
  /// Create new app using SFTP session and CLI args
  pub fn from(sess: &Session, sftp: &Sftp, args: clap::ArgMatches) -> Self {
    let buf = AppBuf::from(sess);
    let state = AppState::default();
    let show_help = args.is_present("shortcuts");
    let show_hidden = args.is_present("all");
    let content = AppContent::from(&buf, sftp, show_hidden);

    Self {
      buf,
      content,
      state,
      show_help,
      show_hidden,
    }
  }

  /// Updates the `AppBuf.local`, `AppContent.local` and `AppState.local`,
  /// using the currently selected item as a PathBuf, the contents of which will
  /// be read into `AppContent.local` while the PathBuf itself will be saved as
  /// `AppBuf.local`. `AppState.local` is reset to `Some(0)`.
  pub fn cd_into_local(&mut self) {
    let i = self.state.local.selected().unwrap_or(0);
    // fix panic if you delete some of the items in your directory
    if self.content.local.is_empty() {
      return;
    }
    self.buf.local.push(&self.content.local[i]);
    if !self.buf.local.is_dir() {
      self.buf.local.pop();
      return;
    }
    self.content.update_local(&self.buf.local, self.show_hidden);
    self.state.local.select(Some(0));
  }

  /// Changes `AppBuf.local` to its parent, and reads the new `PathBuf`'s contents to
  /// `AppContent.local`.
  pub fn cd_out_of_local(&mut self) {
    self.buf.local.pop();
    self.content.update_local(&self.buf.local, self.show_hidden);
    self.state.local.select(Some(0));
  }

  /// Updates the `AppBuf.remote`, `AppContent.remote` and `AppState.remote`,
  /// using the currently selected item as a PathBuf, the contents of which will
  /// be read into `AppContent.remote` while the PathBuf itself will be saved as
  /// `AppBuf.remote`. `AppState.remote` is reset to `Some(0)`.
  pub fn cd_into_remote(&mut self, sftp: &Sftp) {
    // return if dir is empty, or push below will panic
    if self.content.remote.is_empty() {
      return;
    }
    // because this unwrap never fails ⬇
    let i = self.state.remote.selected().unwrap();
    self.buf.remote.push(&self.content.remote[i]);
    // we have to make sure we don't treat files as if they're directories -
    // this functions exactly like `if !self.buf.local.is_dir() {...}` in `cd_into_local`
    if sftp.opendir(self.buf.remote.as_path()).is_err() {
      self.buf.remote.pop();
      return;
    }
    self
      .content
      .update_remote(sftp, &self.buf.remote, self.show_hidden);
    self.state.remote.select(Some(0));
  }

  /// Changes `AppBuf.remote` to its parent, and reads the new `PathBuf`'s contents to
  /// `AppContent.remote`.
  pub fn cd_out_of_remote(&mut self, sftp: &Sftp) {
    self.buf.remote.pop();
    self
      .content
      .update_remote(sftp, &self.buf.remote, self.show_hidden);
    self.state.remote.select(Some(0));
  }
}
