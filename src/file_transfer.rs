//! File transfer utils
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use ssh2::{Session, Sftp};

use crate::app::App;
use crate::app_utils;

/// Download from remote host - directories are downloaded recursively.
pub fn download(sess: &Session, app: &App) -> Result<(), Box<dyn Error>> {
    let sftp = sess.sftp()?;
    let i = app.state.remote.selected().unwrap();
    let source = app.buf.remote.join(&app.content.remote[i]);
    let target = app.buf.local.join(&app.content.remote[i]);
    let mut f = sftp.open(source.as_path())?;
    if f.stat().expect("no stats").is_file() {
        download_file(&mut f, &target)?;
    } else {
        download_directory_recursive(&sftp, &source, &target)?;
    }

    Ok(())
}

fn download_file(
    source: &mut ssh2::File,
    target: &PathBuf
) -> Result<(), Box<dyn Error>> {
    let nbytes: u64 = source.stat()?.size.unwrap_or_default();
    let mut buf = Vec::with_capacity(nbytes as usize);
    source.read_to_end(&mut buf)?;
    let mut f = fs::File::create(target.as_path())?;
    f.write_all(&buf)?;

    Ok(())
}

fn download_directory_recursive(
    sftp: &Sftp,
    source: &PathBuf,
    target: &PathBuf
) -> Result<(), Box<dyn Error>> {
    fs::create_dir(&target).unwrap_or(());
    let readdir_info = sftp.readdir(source).unwrap_or_default();
    for (buf, stat) in readdir_info {
        let new_target = target.join(buf.file_name().unwrap());
        if stat.is_dir() {
            download_directory_recursive(sftp, &buf, &new_target)?;
        } else {
            let mut f = sftp.open(buf.as_path())?;
            download_file(&mut f, &new_target)?;
        }
    }

    Ok(())
}

/// Download from remote host - directories are downloaded recursively.
pub fn upload(sess: &Session, app: &App) -> Result<(), Box<dyn Error>> {
    let sftp = sess.sftp()?;
    let i = app.state.local.selected().unwrap();
    let source = app.buf.local.join(&app.content.local[i]);
    let target = app.buf.remote.join(&app.content.local[i]);
    if !source.is_dir() {
        upload_file(&sftp, &source, &target)?;
    } else {
        // TODO: Fix recursive upload function
        upload_directory_recursive(&sftp, &source, &target).unwrap_or(());
    }

    Ok(())
}

fn upload_file(sftp: &Sftp, source: &PathBuf, target: &PathBuf) -> Result<(), Box<dyn Error>> {
    let buf = fs::read(&source)?;
    let mut f = sftp.create(target.as_path())?;
    f.write_all(&buf)?;

    Ok(())
}

fn upload_directory_recursive(
    sftp: &Sftp,
    source: &PathBuf,
    target: &PathBuf
) -> Result<(), Box<dyn Error>> {
    sftp.mkdir(target.as_path(), 0o666)?; 
    let bufs = app_utils::pathbufs(source);
    for buf in &bufs {
        let new_target = target.join(buf.file_name().unwrap());
        if buf.is_dir() {
            upload_directory_recursive(sftp, source, &new_target)?;
        } else {
            upload_file(sftp, buf, &new_target)?;
        }
    }

    Ok(())
}