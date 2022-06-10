//! SFTP utils
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::net::TcpStream;
use ssh2::{Prompt, Session};

use crate::config::{AuthMethod, Config};

/// Establish SFTP session with a password, given as an argument.
pub fn get_session_with_password(password: &str, conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:{}", conf.addr, conf.port))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    sess.userauth_password(&conf.user, password)?;

    Ok(sess)
}

/// Establish SFTP session with a publickey file, given as an argument.
pub fn get_session_with_pubkey_file(conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:{}", conf.addr, conf.port))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    let pubkey = if let Some(pk) = &conf.pubkey {
        Some(pk.as_path())
    } else {
        None
    };
    let passphrase = if let Some(phrase) = &conf.passphrase {
        Some(phrase.as_str())
    } else {
        None
    };
    if let AuthMethod::PrivateKey(sk) = &conf.auth_method {
        let privatekey = Path::new(&sk);
        sess.userauth_pubkey_file(&conf.user, pubkey, privatekey, passphrase)?;
    }

    Ok(sess)
}

#[allow(unreachable_code, unused_variables, unused_mut)]
pub fn get_session_with_keyboard_interactive(conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:{}", conf.addr, conf.port))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    let mut password_prompt = Prompt { 
        text: std::borrow::Cow::Borrowed("Password:"), 
        echo: true 
    };
    // sess.userauth_keyboard_interactive(&conf.user, &mut prompter);

    Ok(sess)
}

/// Establish SFTP session automatically with a userauth agent.
/// With no password or identity file arguments, this is used as the default, and if it fails
/// it will attempt to establish an interactive keyboard session to authenticate.
pub fn get_session_with_userauth_agent(conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:{}", conf.addr, conf.port))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    if let Err(_) = sess.userauth_agent(&conf.user) {
        return get_session_with_keyboard_interactive(conf)
    }

    Ok(sess)
}

/// Supposed to mimic `ls` in a terminal, yielding a list of the contents of a directory.
/// The implied files `.` and `..` are ignored.
pub fn ls(sess: &Session, buf: &PathBuf, show_hidden: bool) -> Vec<String> {
    let mut items: Vec<String> = sess
        .sftp()
        .unwrap()
        .readdir(&buf)
        .unwrap()
        .iter()
        .map(|(buf, _)| buf.file_name().unwrap().to_str().unwrap_or_default().to_string())
        .filter(|s| if show_hidden { true } else { !s.starts_with('.') })
        .collect();
    items.sort();
    items
}

/// Gets the base directory ($HOME) of the remote client, e.g. /home/<user>/
pub fn home_dir(sess: &Session) -> PathBuf {
    let mut channel = sess.channel_session().unwrap();
    channel.exec("pwd").unwrap_or_else(|e| {
        eprintln!("Failure to execute commmand pwd: {e}");
        eprintln!("Perhaps client does not have the permissions to read their own home directory?");
        channel.write(b"ERROR").unwrap();
    });
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap_or_default();
    PathBuf::from(s.strip_suffix('\n').unwrap_or_default())
}