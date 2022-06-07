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

#[allow(unreachable_code)]
pub fn get_session_with_keyboard_interactive(conf: &Config) -> Result<Session, Box<dyn Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:22", conf.addr))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    let mut _prompter = Prompt { 
        text: std::borrow::Cow::Borrowed("Password:"), 
        echo: true 
    };
    unimplemented!("Keyboard interactivity has not been implemented yet");
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

pub fn ls(sess: &Session, buf: &PathBuf, show_hidden: bool) -> Vec<String> {
    let dir_to_read = buf.as_os_str().to_str().unwrap();
    let command = if show_hidden { 
        format!("ls -a {}", dir_to_read) 
    } else { 
        format!("ls {}", dir_to_read)
    };
    let mut channel = sess.channel_session().unwrap();
    channel.exec(&command).unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap_or_default();
    let mut items: Vec<String> = s
        .lines()
        .map(|s| s.to_string())
        .collect();
    items.sort();
    items
}

pub fn pwd(sess: &Session) -> PathBuf {
    let mut channel = sess.channel_session().unwrap();
    channel.exec("pwd").unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap_or_default();
    PathBuf::from(s.strip_suffix('\n').unwrap_or("$HOME"))
}