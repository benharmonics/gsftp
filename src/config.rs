//! App configuration and argument parsing.
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use dns_lookup::lookup_host;
use clap::{arg, Command, ArgMatches};

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

pub fn args() -> ArgMatches {
    Command::new(PROGRAM_NAME)
        .author("benharmonics")
        .version("0.1.0")
        .about("Secure file transfer tool with graphical interface")
        .arg(arg!(<DESTINATION> "Required remote connection, e.g. username@host"))
        .arg(arg!(-s --privatekey "Path to private key file").number_of_values(1).conflicts_with("password"))
        .arg(arg!(-P --pubkey "Path to public key file").number_of_values(1).requires("privatekey"))
        .arg(arg!(--passphrase "SSH additional passphrase").number_of_values(1).requires("pubkey"))
        .arg(arg!(-p --password "Input SSH password for remote server").number_of_values(1).conflicts_with("privatekey"))
        .arg(arg!(-a --agent "Authenticate with SSH agent").default_value("true").takes_value(false).conflicts_with_all(&["privatekey", "password"]))
        .arg(arg!(-f --fullscreen "Fullscreen mode (without help panel)").takes_value(false))
        .get_matches()
}

#[derive(Debug)]
/// There are several principle authentication methods for SSH.
/// Implicitly, if all authentication methods fail, the program will default to asking the
/// user to input their authentication details manually.
/// ^^^ NOT IMPLEMENTED
pub enum AuthMethod {
    Password(String),
    PrivateKey(String),
    Agent,
}

#[derive(Debug)]
/// Our `Config` struct keeps track of our SFTP destination user@addr,
/// as well as some other application configuration info.
pub struct Config {
    pub user: String,
    pub addr: String,
    pub fullscreen: bool,
    pub auth_method: AuthMethod,
    pub pubkey: Option<Box<PathBuf>>,
    pub passphrase: Option<Box<String>>,
}

impl Config {
    pub fn from(args: ArgMatches) -> Config {
        // The program takes a destination as input in the format username@host, typically something like
        // user@10.0.0.8 on a LAN. We parse this input as follows:
        // If the user input a hostname as an IP Address, we can just parse it as such - easy!
        // Otherwise, we're going to have to try to use DNS to resolve the hostname into an IP address.
        // If both of these options fail, we'll just have to yield an error message and close the program.
        let conn: Vec<&str> = args
            .value_of("DESTINATION")
            .unwrap()
            .split("@")
            .collect();
        let user = String::from(conn[0]);
        let addr = if let Ok(ip) = conn[1].parse::<Ipv4Addr>() {
            ip.to_string()
        } else {
            lookup_host(conn[1])
                .unwrap_or_default()
                .get(1)
                .unwrap_or_else(|| {
                    eprintln!("Couldn't resolve remote server {}.", conn[1]);
                    eprintln!("Example usage: {} user@192.168.0.8", PROGRAM_NAME);
                    std::process::exit(1);
                })
                .to_string()
        };
        let fullscreen = args.is_present("fullscreen");
        // TODO: change this to a match statement to catch all possible arms?
        let auth_method = if args.is_present("password") {
            AuthMethod::Password(String::from(args.value_of("password").unwrap()))
        } else if args.is_present("privatekey") {
            AuthMethod::PrivateKey(String::from(args.value_of("privatekey").unwrap()))
        } else {
            AuthMethod::Agent
        };
        let pk_path = Path::new(args.value_of("pubkey").unwrap_or_default());
        let pubkey = if pk_path.exists() {
            Some(Box::new(pk_path.to_owned()))
        } else { None };
        let passphrase = if let Some(passphrase) = args.value_of("passphrase") {
            Some(Box::new(passphrase.to_string()))
        } else { None };

        Config { user, addr, fullscreen, auth_method, pubkey, passphrase, }
    }
}