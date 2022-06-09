//! Utils to read the contents of local and remote directories
use std::{env, fs, path::PathBuf};
use ssh2::Session;

use crate::sftp;

#[derive(Debug)]
/// Contains the contents of our current working directories as `Vec<String>`.
pub struct AppContent {
    pub local: Vec<String>,
    pub remote: Vec<String>,
}

#[derive(Debug)]
/// The `AppBuf` struct contains our working directories, both local and remote, as PathBufs.
pub struct AppBuf {
    pub local: PathBuf,
    pub remote: PathBuf,
}

impl From<&mut Session> for AppBuf {
    /// Yields a `AppBuf` with the `local` field defaulting to the current working directory;
    /// the `remote` field defaults to the remote connection's home directory (e.g. /home/$USER).
    fn from(sess: &mut Session) -> AppBuf {
        let local = env::current_dir().unwrap_or_else(|e| {
            eprintln!("Fatal error reading current directory: {e}");
            std::process::exit(1);
        });
        let remote = sftp::home_dir(sess);
        AppBuf { local, remote }
    }
}

impl AppContent {
    /// The `AppContent` struct holds two vectors which contain the contents of the local and remote
    /// directories contained by the `PathBuf` directories in the `AppBuf` struct
    /// the `remote` field defaults to the remote connection's home directory (e.g. /home/$USER).
    pub fn from(buf: &AppBuf, sess: &Session, show_hidden: bool) -> AppContent {
        let mut local: Vec<String> = pathbufs(&buf.local)
            .iter()
            .map(|b| b.file_name().unwrap_or_default().to_str().unwrap_or_default())
            .filter(|s| !s.is_empty())
            .filter(|s| if !show_hidden { !s.starts_with('.') } else { true })
            .map(|s| s.to_string())
            .collect();
        local.sort();
        let remote = sftp::ls(sess, &buf.remote, show_hidden);
        AppContent { local, remote }
    }

    /// Given the current `AppBuf.local`, updates the `AppContent.local` 
    /// to reflect the current local dir's contents.
    pub fn update_local(&mut self, buf: &PathBuf, show_hidden: bool) {
        self.local = pathbufs(buf)
            .iter()
            .map(|b| b.file_name().unwrap_or_default().to_str().unwrap_or_default())
            .filter(|s| !s.is_empty())
            .filter(|s| if !show_hidden { !s.starts_with('.') } else { true })
            .map(|s| s.to_string())
            .collect();
        self.local.sort();
    }

    /// Given the current `AppBuf.remote`, updates the `AppContent.remote` 
    /// to reflect the current remote dir's contents.
    pub fn update_remote(&mut self, sess: &Session, buf:&PathBuf, show_hidden: bool) {
        self.remote = sftp::ls(sess, buf, show_hidden);
    }
}

fn pathbufs(buf: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(buf)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .map(|res| res.unwrap_or_default())
        .filter(|buf| buf.exists())
        .collect()
}