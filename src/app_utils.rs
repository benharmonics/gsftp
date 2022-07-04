//! Utils to read the contents of local and remote directories
use ssh2::{Session, Sftp};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use tui::widgets::ListState;

use crate::sftp;

#[derive(Debug)]
/// The `AppBuf` struct contains our working directories, both local and remote, as PathBufs.
pub struct AppBuf {
    pub local: PathBuf,
    pub remote: PathBuf,
}

impl From<&Session> for AppBuf {
    /// Yields a `AppBuf` with the `local` field defaulting to the current working directory;
    /// the `remote` field defaults to the remote connection's home directory (e.g. /home/$USER).
    fn from(sess: &Session) -> AppBuf {
        let local = env::current_dir().unwrap_or_else(|e| {
            eprintln!("Fatal error reading current directory: {e}");
            std::process::exit(1);
        });
        let remote = sftp::home_dir(sess);
        AppBuf { local, remote }
    }
}

#[derive(Debug)]
/// Contains the contents of our current working directories as `Vec<String>`.
pub struct AppContent {
    pub local: Vec<String>,
    pub remote: Vec<String>,
}

impl AppContent {
    /// The `AppContent` struct holds two vectors which contain the contents of the local and remote
    /// directories contained by the `PathBuf` directories in the `AppBuf` struct
    /// the `remote` field defaults to the remote connection's home directory (e.g. /home/$USER).
    pub fn from(buf: &AppBuf, sftp: &Sftp, show_hidden: bool) -> AppContent {
        let local = sort_and_stringify(read_dir_contents(&buf.local), show_hidden);
        let remote = sftp::ls(sftp, &buf.remote, show_hidden);
        AppContent { local, remote }
    }

    /// Given the current `AppBuf.local`, updates the `AppContent.local`
    /// to reflect the current local dir's contents.
    pub fn update_local(&mut self, buf: &PathBuf, show_hidden: bool) {
        self.local = sort_and_stringify(read_dir_contents(buf), show_hidden);
    }

    /// Given the current `AppBuf.remote`, updates the `AppContent.remote`
    /// to reflect the current remote dir's contents.
    pub fn update_remote(&mut self, sftp: &Sftp, buf: &Path, show_hidden: bool) {
        self.remote = sftp::ls(sftp, buf, show_hidden);
    }
}

pub fn read_dir_contents(buf: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(buf)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .map(|res| res.unwrap_or_default())
        .filter(|buf| buf.exists())
        .collect()
}

fn sort_and_stringify(bufs: Vec<PathBuf>, show_hidden: bool) -> Vec<String> {
    let mut bufs: Vec<String> = bufs
        .iter()
        .map(|b| {
            b.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
        })
        .filter(|s| !s.is_empty())
        .filter(|s| {
            if !show_hidden {
                !s.starts_with('.')
            } else {
                true
            }
        })
        .map(|s| s.to_string())
        .collect();
    bufs.sort_by(|s1, s2| s1.to_lowercase().partial_cmp(&s2.to_lowercase()).unwrap());
    bufs
}

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
    pub fn default() -> AppState {
        let mut local = ListState::default();
        let mut remote = ListState::default();
        local.select(Some(0));
        remote.select(Some(0));
        let active = ActiveState::Local;

        AppState {
            local,
            remote,
            active,
        }
    }
}
