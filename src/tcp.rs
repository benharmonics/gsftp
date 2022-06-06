use std::error;
use std::path::PathBuf;
use std::io::Read;
use std::net::TcpStream;
use ssh2::Session;

use crate::config::Config;

pub fn get_session_with_password(password: &str, conf: &Config) -> Result<Session, Box<dyn error::Error>> {
    let mut sess = Session::new()?;
    let stream = TcpStream::connect(format!("{}:22", conf.addr))?;
    sess.set_tcp_stream(stream);
    sess.handshake()?;
    sess.userauth_password(&conf.user, password)?;

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