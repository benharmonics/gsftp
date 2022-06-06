use std::{env, fs, path::PathBuf};

pub struct DirContents {
    pub local: Vec<String>,
    pub remote: Vec<String>,
}
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
}

impl DirContents {
    pub fn from(buf: &DirBuf) -> DirContents {
        let local: Vec<String> = pathbufs(&buf.local)
            .iter()
            .map(|b| b.file_name().unwrap_or_default().to_str().unwrap_or_default())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        // let remote: Vec<String> = pathbufs(&buf.remote)
        //     .iter()
        //     .map(|b| b.file_name().unwrap_or_default().to_str().unwrap_or_default())
        //     .filter(|s| s.is_empty())
        //     .map(|s| s.to_string())
        //     .collect();
        let remote = vec![];
        DirContents { local, remote }
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