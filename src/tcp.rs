//! TCP/SFTP utils.
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::net::TcpStream;
use ssh2::{Prompt, Session};

use crate::config::{AuthMethod, Config};

pub fn get_session_with_password(password: &str, conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:22", conf.addr))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    sess.userauth_password(&conf.user, password)?;

    Ok(sess)
}

pub fn get_session_with_pubkey_file(conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:22", conf.addr))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    let pubkey = if let Some(pk) = &conf.pubkey {
        Some(pk.as_path())
    } else { None };
    let passphrase = None;
    if let AuthMethod::PrivateKey(sk) = &conf.auth_method {
        let privatekey = Path::new(&sk);
        sess.userauth_pubkey_file(&conf.user, pubkey, privatekey, passphrase)?;
    }

    Ok(sess)
}

pub fn get_session_with_keyboard_interactive(conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:22", conf.addr))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    let mut _prompter = Prompt { 
        text: std::borrow::Cow::Borrowed("Password:"), 
        echo: true 
    };
    //sess.userauth_keyboard_interactive(&conf.user, &mut prompter);

    Ok(sess)
}

pub fn get_session_with_userauth_agent(conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:22", conf.addr))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    if let Err(_) = sess.userauth_agent(&conf.user) {
        return get_session_with_keyboard_interactive(conf)
    }

    Ok(sess)
}

pub fn ls(sess: &Session) -> Vec<String> {
    let mut channel = sess.channel_session().unwrap();
    channel.exec("ls").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap_or_default();
    channel.wait_close().unwrap();
    s.lines().map(|s| s.to_string()).collect::<Vec<String>>()
}

pub fn pwd(sess: &Session) -> PathBuf {
    let mut channel = sess.channel_session().unwrap();
    channel.exec("pwd").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap_or_default();
    channel.wait_close().unwrap();
    PathBuf::from(s)
}