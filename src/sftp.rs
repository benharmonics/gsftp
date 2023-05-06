//! SFTP utils
use ssh2::{Prompt, Session, Sftp};
use std::{
  error::Error,
  io::{Read, Write},
  net::{SocketAddr, TcpStream},
  path::{Path, PathBuf},
  str::FromStr,
  time::Duration,
};

use crate::config::Config;

/// Establish SFTP session with a password, given as an argument
pub fn get_session_with_password(password: &str, conf: &Config) -> Result<Session, Box<dyn Error>> {
  let mut sess = Session::new()?;
  let addr = SocketAddr::from_str(format!("{}:{}", conf.addr, conf.port).as_str())?;
  let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(5000))?;
  sess.set_tcp_stream(stream);
  sess.handshake()?;
  sess.userauth_password(&conf.user, password)?;

  Ok(sess)
}

/// Establish SFTP session with a public key file, given as an argument
pub fn get_session_with_pubkey_file(sk: &str, conf: &Config) -> Result<Session, Box<dyn Error>> {
  let mut sess = Session::new()?;
  let addr = SocketAddr::from_str(format!("{}:{}", conf.addr, conf.port).as_str())?;
  let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(7000))?;
  sess.set_tcp_stream(stream);
  sess.handshake()?;
  let private_key = Path::new(sk);
  let pubkey = conf.pubkey.as_deref();
  let passphrase = conf.passphrase.as_deref();
  sess.userauth_pubkey_file(&conf.user, pubkey, private_key, passphrase)?;

  Ok(sess)
}

#[allow(unreachable_code, unused_variables, unused_mut)]
/// Gets credentials via an interactive prompt
/// (NOT IMPLEMENTED)
pub fn get_session_with_keyboard_interactive(conf: &Config) -> Result<Session, Box<dyn Error>> {
  let mut sess = Session::new()?;
  let addr = SocketAddr::from_str(format!("{}:{}", conf.addr, conf.port).as_str())?;
  let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(5000))?;
  sess.set_tcp_stream(stream);
  sess.handshake()?;
  let mut password_prompt = Prompt {
    text: std::borrow::Cow::Borrowed("Password:"),
    echo: true,
  };
  // sess.user_auth_keyboard_interactive(&conf.user, &mut prompter);

  Ok(sess)
}

/// Establish SFTP session automatically with a user auth agent.
/// With no password or identity file arguments, this is used as the default; if it fails
/// it will attempt to establish an interactive keyboard session to authenticate (not implemented).
pub fn get_session_with_user_auth_agent(conf: &Config) -> Result<Session, Box<dyn Error>> {
  let mut sess = Session::new()?;
  let addr = SocketAddr::from_str(format!("{}:{}", conf.addr, conf.port).as_str())?;
  let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(5000))?;
  sess.set_tcp_stream(stream);
  sess.handshake()?;
  if sess.userauth_agent(&conf.user).is_err() {
    return get_session_with_keyboard_interactive(conf);
  }

  Ok(sess)
}

/// Mimics the behavior of `ls` in a terminal, yielding the contents of a directory.
/// The implied files `.` and `..` are ignored.
pub fn ls(sftp: &Sftp, buf: &Path, show_hidden: bool) -> Vec<String> {
  let mut items: Vec<String> = sftp
    .readdir(buf)
    .unwrap_or_default()
    .iter()
    .filter_map(|(buf, _)| buf.file_name())
    .map(|s| s.to_str().unwrap_or_default().to_string())
    .filter(|s| show_hidden || !s.starts_with('.'))
    .collect();
  items.sort_by(|s1, s2| s1.to_lowercase().partial_cmp(&s2.to_lowercase()).unwrap());
  items
}

/// Gets the base directory ($HOME) of the remote client, i.e. `/home/user/` on Linux
/// or `C:\Users\user` on Windows
pub fn home_dir(sess: &Session) -> PathBuf {
  let mut channel = sess.channel_session().unwrap();
  channel.exec("pwd").unwrap_or_else(|e| {
    eprintln!("Failure to execute command pwd: {e}");
    eprintln!("Perhaps client does not have the permissions to read their own home directory?");
    channel.write_all(b"ERROR").unwrap();
  });
  let mut s = String::new();
  channel.read_to_string(&mut s).unwrap_or_default();
  PathBuf::from(s.strip_suffix('\n').unwrap_or_default())
}
