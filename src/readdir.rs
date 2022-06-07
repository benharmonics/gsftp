//! Utils to read the contents of local directories into convenient data structures.
use std::{env, fs, path::PathBuf};
use ssh2::Session;

use crate::tcp;

#[derive(Debug)]
pub struct DirContents {
    pub local: Vec<String>,
    pub remote: Vec<String>,
}

#[derive(Debug)]
pub struct DirBuf {
    pub local: PathBuf,
    pub remote: PathBuf,
}

impl DirBuf {
    pub fn default() -> DirBuf {
        let local = env::current_dir().unwrap_or_else(|e| {
            eprintln!("Fatal error reading current directory: {e}");
            std::process::exit(1);
        });
        let remote = PathBuf::default();
        DirBuf { local, remote }
    }
    
    pub fn new(sess: &mut Session) -> DirBuf {
        let local = env::current_dir().unwrap_or_else(|e| {
            eprintln!("Fatal error reading current directory: {e}");
            std::process::exit(1);
        });
        let remote = tcp::pwd(sess);
        DirBuf { local, remote }
    }
}

impl DirContents {
    pub fn from(buf: &DirBuf, sess: &Session, show_hidden: bool) -> DirContents {
        let mut local: Vec<String> = pathbufs(&buf.local)
            .iter()
            .map(|b| b.file_name().unwrap_or_default().to_str().unwrap_or_default())
            .filter(|s| !s.is_empty())
            .filter(|s| if !show_hidden { !s.starts_with('.') } else { true })
            .map(|s| s.to_string())
            .collect();
        local.sort();
        let remote = tcp::ls(sess);
        DirContents { local, remote }
    }

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
}

fn pathbufs(buf: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(buf)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .map(|res| res.unwrap_or_default())
        .filter(|buf| buf.exists())
        .collect()
}