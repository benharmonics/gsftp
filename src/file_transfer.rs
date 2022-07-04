//! File transfer utils
use ssh2::{Session, Sftp};
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::app::App;
use crate::app_utils;

/// Download currently selected item from remote host - directories are downloaded recursively
pub fn download(from: &Path, to: &Path, sftp: Sftp) -> Result<(), Box<dyn Error>> {
    let mut f = sftp.open(from)?;
    if f.stat().expect("no stats").is_file() {
        download_file(&mut f, &from)?;
    } else {
        download_directory_recursive(from, to, &sftp)?;
    }

    Ok(())
}

fn download_file(file: &mut ssh2::File, target: &Path) -> Result<(), Box<dyn Error>> {
    if let Ok(mut f) = fs::File::create(target) {
        let n_bytes: u64 = file.stat()?.size.unwrap();
        let mut buf = Vec::with_capacity(n_bytes as usize);
        file.read_to_end(&mut buf)?;
        f.write_all(&buf)?;
    }

    Ok(())
}

fn download_directory_recursive(
    from: &Path,
    to: &Path,
    sftp: &Sftp,
) -> Result<(), Box<dyn Error>> {
    if fs::create_dir(&to).is_ok() {
        let readdir_info = sftp.readdir(from).unwrap_or_default();
        for (buf, stat) in readdir_info {
            if stat.file_type().is_symlink() {
                continue;
            }
            let new_target = to.join(buf.file_name().unwrap());
            if stat.is_dir() {
                download_directory_recursive(&buf, &new_target, sftp,)?;
            } else {
                let mut f = sftp.open(buf.as_path())?;
                download_file(&mut f, &new_target)?;
            }
        }
    }

    Ok(())
}

/// Upload currently selected item to remote host - directories are uploaded recursively
pub fn upload(sess: &Session, app: &App) -> Result<(), Box<dyn Error>> {
    let sftp = sess.sftp()?;
    let i = app.state.local.selected().unwrap();
    let source = app.buf.local.join(&app.content.local[i]);
    let target = app.buf.remote.join(&app.content.local[i]);
    if source.is_dir() {
        upload_directory_recursive(sess, &sftp, &source, &target)?;
    } else {
        upload_file(&sftp, &source, &target)?;
    }

    Ok(())
}

fn upload_file(sftp: &Sftp, source: &PathBuf, target: &Path) -> Result<(), Box<dyn Error>> {
    if let Ok(mut f) = sftp.create(target) {
        let buf = fs::read(&source)?;
        f.write_all(&buf)?;
    }

    Ok(())
}

fn upload_directory_recursive(
    sess: &Session,
    sftp: &Sftp,
    source: &PathBuf,
    target: &Path,
) -> Result<(), Box<dyn Error>> {
    // TODO: try and make this more platform-agnostic
    let mut channel = sess.channel_session()?;
    let target_str = target.to_str().unwrap();
    let command = format!("mkdir '{target_str}'");
    channel.exec(&command)?;
    // sftp.mkdir(target, 0o644)?;
    for buf in &app_utils::read_dir_contents(source) {
        if buf.is_symlink() {
            continue;
        }
        let new_target = target.join(buf.file_name().unwrap());
        if buf.is_dir() {
            upload_directory_recursive(sess, sftp, buf, &new_target)?;
        } else {
            // It can take a second for the remote connection to actually make the directory...
            for _ in 0..5 {
                if sftp.opendir(new_target.parent().unwrap()).is_err() {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                break;
            }
            upload_file(sftp, buf, &new_target)?;
        }
    }

    Ok(())
}
