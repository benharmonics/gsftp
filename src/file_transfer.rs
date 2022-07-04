//! File transfer utils
use ssh2::{Session, Sftp};
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::app_utils;

/// Download currently selected item from remote host - directories are downloaded recursively
pub fn download(from: &Path, to: &Path, sftp: &Sftp) -> Result<(), Box<dyn Error>> {
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

fn download_directory_recursive(from: &Path, to: &Path, sftp: &Sftp) -> Result<(), Box<dyn Error>> {
    if let Ok(_) = fs::create_dir(&to) {
        let readdir_info = sftp.readdir(from).unwrap_or_default();
        for (buf, stat) in readdir_info {
            if stat.file_type().is_symlink() {
                continue;
            }
            let new_target = to.join(buf.file_name().unwrap());
            if stat.is_dir() {
                download_directory_recursive(&buf, &new_target, sftp)?;
            } else {
                let mut f = sftp.open(buf.as_path())?;
                download_file(&mut f, &new_target)?;
            }
        }
    }

    Ok(())
}

/// Upload currently selected item to remote host - directories are uploaded recursively
pub fn upload(from: &Path, to: &Path, sess: &Session, sftp: &Sftp) -> Result<(), Box<dyn Error>> {
    if from.is_dir() {
        upload_directory_recursive(from, to, sess, sftp)?;
    } else {
        upload_file(from, to, sftp)?;
    }

    Ok(())
}

fn upload_file(from: &Path, to: &Path, sftp: &Sftp) -> Result<(), Box<dyn Error>> {
    if let Ok(mut f) = sftp.create(to) {
        let buf = fs::read(&from)?;
        f.write_all(&buf)?;
    }

    Ok(())
}

fn upload_directory_recursive(
    from: &Path,
    to: &Path,
    sess: &Session,
    sftp: &Sftp,
) -> Result<(), Box<dyn Error>> {
    // TODO: try and make this more platform-agnostic
    let mut channel = sess.channel_session()?;
    let command = format!("mkdir '{}'", to.to_str().unwrap());
    channel.exec(&command)?;
    // sftp.mkdir(to, 0o644)?;
    for buf in &app_utils::read_dir_contents(&from.to_path_buf()) {
        if buf.is_symlink() {
            continue;
        }
        let new_target_buf = to.join(buf.file_name().unwrap());
        if buf.is_dir() {
            upload_directory_recursive(buf, &new_target_buf, sess, sftp)?;
        } else {
            // It can take a second for the remote connection to actually make the directory...
            for _ in 0..5 {
                if let Err(_) = sftp.opendir(new_target_buf.parent().unwrap()) {
                    // TODO: This is a bad way to handle this.
                    thread::sleep(Duration::from_millis(5));
                    continue;
                }
                break;
            }
            upload_file(buf, &new_target_buf, sftp)?;
        }
    }

    Ok(())
}
