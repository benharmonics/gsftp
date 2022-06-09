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
        download_file_from_remote(sess, &mut f, &target)?;
    }
    // let readdir_info = sftp.readdir(&source).unwrap_or_default();
    // let children_of_source: Vec<&PathBuf> = readdir_info.iter().map(|(buf, _)| buf).collect();
    // for (buf, stat) in &readdir_info {
    //     if stat.is_file() {
    //         download_file_from_remote(sess, &source, &target)?;
    //     } else {
    //         download_directory_from_remote_recursive(sess, &sftp, buf, &target)?;
    //     }
    // }

    Ok(())
}

fn download_file_from_remote(sess: &Session, source: &mut ssh2::File, target: &PathBuf) -> Result<(), Box<dyn Error>> {
    let nbytes = &source.stat().expect("Failed to get file metatdata").size.unwrap();
    let mut buffer = Vec::with_capacity(*nbytes as usize);
    source.read_to_end(&mut buffer)?;
    // let mut s = String::new();
    // source.read_to_string(&mut s)?;
    // write!(f, "{s}")?;
    let mut f = fs::File::create(target.as_path()).unwrap();
    f.write_all(&buffer).expect("Failed to write");

    Ok(())
}

fn download_directory_from_remote_recursive(
    sess: &Session,
    sftp: &Sftp,
    buf: &PathBuf,
    target: &PathBuf
) -> Result<(), Box<dyn Error>> {
    fs::create_dir(&target).unwrap_or(());
    let readdir_info = sftp.readdir(buf).unwrap_or_default();
    for (buf, stat) in readdir_info {

    }

    Ok(())
}