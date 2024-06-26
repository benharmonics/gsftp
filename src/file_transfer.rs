//! File transfer utils
use ssh2::{Session, Sftp};
use std::error::Error;
use std::fmt::{self, Formatter};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use std::{fs, io};

use crate::{app::App, app_utils};

enum TransferKind {
  Upload,
  Download,
}

#[derive(Debug)]
pub struct TransferError {
  message: String,
}

impl fmt::Display for TransferError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl From<Box<dyn Error>> for TransferError {
  fn from(error: Box<dyn Error>) -> TransferError {
    let message = format!("TRANSFER ERROR: {}", error);
    TransferError { message }
  }
}

/// The File tranfer API struct we'll call from main.rs.
/// We keep track of the source path, destination path, and whether the
/// transfer is an upload or a download.
pub struct Transfer {
  from: PathBuf,
  to: PathBuf,
  kind: TransferKind,
  sess: Session,
  sftp: Sftp,
}

impl Transfer {
  /// Create a new upload transfer, ready to be executed
  pub fn upload(app: &App, sess: &Session) -> Self {
    let i = app.state.local.selected().unwrap();
    let from = app.buf.local.join(&app.content.local[i]);
    let to = app.buf.remote.join(&app.content.local[i]);
    let kind = TransferKind::Upload;

    // TODO: get ride of clone
    let sess = sess.clone();
    let sftp = sess.sftp().expect("Failed to create SFTP session.");

    Self {
      from,
      to,
      kind,
      sess,
      sftp,
    }
  }

  /// Create a new download transfer, ready to be executed
  pub fn download(app: &App, sess: &Session) -> Self {
    let i = app.state.remote.selected().unwrap();
    let from = app.buf.remote.join(&app.content.remote[i]);
    let to = app.buf.local.join(&app.content.remote[i]);
    let kind = TransferKind::Download;

    // TODO: get ride of clone
    let sess = sess.clone();
    let sftp = sess.sftp().expect("Failed to create SFTP session.");

    Self {
      from,
      to,
      kind,
      sess,
      sftp,
    }
  }

  /// Execute a transfer through an SSH session (either upload or download the file)
  pub fn execute(self) -> Result<(), TransferError> {
    let action = match self.kind {
      TransferKind::Download => download(&self, &self.sftp),
      TransferKind::Upload => upload(&self, &self.sess, &self.sftp),
    };
    if let Err(e) = action {
      return Err(TransferError::from(e));
    }

    Ok(())
  }
}

// Download currently selected item from remote host - directories are downloaded recursively
fn download(transfer: &Transfer, sftp: &Sftp) -> Result<(), Box<dyn Error>> {
  let from = transfer.from.as_path();
  let to = transfer.to.as_path();
  let mut remote_file = sftp.open(from)?;
  if remote_file.stat().expect("no stats").is_file() {
    download_file(&mut remote_file, to)?;
  } else {
    download_directory_recursive(from, to, sftp)?;
  }

  Ok(())
}

fn download_file(remote_file: &mut ssh2::File, to: &Path) -> Result<(), Box<dyn Error>> {
  // "create" opens a file in write-only mode
  if let Ok(mut local_file) = fs::File::create(to) {
    let n_bytes: u64 = remote_file.stat()?.size.unwrap_or_default();
    let mut buf = Vec::with_capacity(n_bytes as usize);
    remote_file.read_to_end(&mut buf)?; // read contents into buf
    local_file.write_all(&buf)?; // write contents from buf
  }

  Ok(())
}

fn download_directory_recursive(from: &Path, to: &Path, sftp: &Sftp) -> Result<(), Box<dyn Error>> {
  if fs::create_dir(to).is_ok() {
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

// Upload currently selected item to remote host - directories are uploaded recursively
fn upload(transfer: &Transfer, sess: &Session, sftp: &Sftp) -> Result<(), Box<dyn Error>> {
  let from = transfer.from.as_path();
  let to = transfer.to.as_path();
  if from.is_dir() {
    upload_directory_recursive(from, to, sess, sftp)?;
  } else {
    upload_file(from, to, sftp)?;
  }

  Ok(())
}

fn upload_file(from: &Path, to: &Path, sftp: &Sftp) -> Result<(), io::Error> {
  if let Ok(mut remote_file) = sftp.create(to) {
    let buf = fs::read(from).unwrap_or_default();
    remote_file.write_all(&buf)?;
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
  for buf in &app_utils::read_dir_contents(from) {
    if buf.is_symlink() {
      continue;
    }
    let new_target_buf = to.join(buf.file_name().unwrap_or_default());
    if buf.is_dir() {
      upload_directory_recursive(buf, &new_target_buf, sess, sftp)?;
    } else {
      // It can take a second for the remote connection to actually make the directory...
      for _ in 0..5 {
        if sftp.opendir(new_target_buf.parent().unwrap()).is_err() {
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
