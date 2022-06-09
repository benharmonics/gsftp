//! File transfer utils
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use ssh2::{Session, Sftp};

use crate::app::App;

pub fn download_from_remote(sess: &Session, app: &App) -> Result<(), Box<dyn Error>> {
    let sftp = sess.sftp()?;
    let i = app.state.remote.selected().unwrap();
    let source = app.buf.remote.join(&app.content.remote[i]);
    let target = app.buf.local.join(&app.content.remote[i]);
    let mut f = sftp.open(source.as_path()).expect("Failed to open file");
    if f.stat().expect("no stats").is_file() {
        download_file_from_remote(&mut f, &target)?;
    } else {
        download_directory_from_remote_recursive(&sftp, &source, &target)?;
    }

    Ok(())
}

fn download_file_from_remote(
    source: &mut ssh2::File,
    target: &PathBuf
) -> Result<(), Box<dyn Error>> {
    let nbytes: u64 = source.stat()?.size.unwrap_or_default();
    let mut f = fs::File::create(target.as_path())?;
    let mut buf = Vec::with_capacity(nbytes as usize);
    source.read_to_end(&mut buf)?;
    f.write_all(&buf)?;

    Ok(())
}

fn download_directory_from_remote_recursive(
    sftp: &Sftp,
    source: &PathBuf,
    target: &PathBuf
) -> Result<(), Box<dyn Error>> {
    fs::create_dir(&target).unwrap_or(());
    let readdir_info = sftp.readdir(source).unwrap_or_default();
    for (buf, stat) in readdir_info {
        let new_target = target.join(buf.file_name().unwrap());
        if stat.is_dir() {
            download_directory_from_remote_recursive(sftp, &buf, &new_target)?;
        } else {
            let mut f = sftp.open(buf.as_path())?;
            download_file_from_remote(&mut f, &new_target)?;
        }
    }

    Ok(())
}