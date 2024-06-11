//! SFTP configuration and argument parsing
use clap::{arg, ArgMatches, Command};
use dns_lookup::lookup_host;
use ssh2::{KeyboardInteractivePrompt, Prompt};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process;

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

pub fn args() -> ArgMatches {
    Command::new(PROGRAM_NAME)
        .author("benharmonics")
        .version("0.1.0")
        .about("Secure file transfer tool with graphical interface")
        .before_help("https://github.com/benharmonics/gsftp/")
        .arg(arg!(<DESTINATION> "Required remote connection, e.g. username@host"))
        .arg(arg!(-a --all "Show hidden files").takes_value(false))
        .arg(
            arg!(-i --identity "Authenticate with identity file, i.e. private key (recommended)")
                .number_of_values(1)
                .conflicts_with_all(&["password", "agent"]),
        )
        .arg(
            arg!(-A --agent "Authenticate with SSH agent")
                .default_value("on")
                .takes_value(false)
                .conflicts_with_all(&["password", "identity"]),
        )
        .arg(
            arg!(--password "Authenticate with password (not recommended)")
                .number_of_values(1)
                .conflicts_with_all(&["agent", "identity"]),
        )
        .arg(
            arg!(--pubkey "Public key file")
                .number_of_values(1)
                .requires("identity"),
        )
        .arg(
            arg!(--passphrase "Additional passphrase")
                .number_of_values(1)
                .requires("identity"),
        )
        .arg(
            arg!(-P --port "SSH port")
                .default_value("22")
                .takes_value(true),
        )
        // .arg(
        //     arg!(--manual "NOT IMPLEMENTED")
        //         .takes_value(false)
        //         .conflicts_with_all(&["password", "identity", "agent"]),
        // )
        .arg(arg!(--shortcuts "Start with keyboard shortcut help panel open").takes_value(false))
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
    Manual,
}

#[derive(Debug)]
/// Static, immutable SFTP configuration
pub struct Config {
    pub user: String,
    pub addr: String,
    pub auth_method: AuthMethod,
    pub pubkey: Option<PathBuf>,
    pub passphrase: Option<String>,
    pub port: u16,
}

impl From<&ArgMatches> for Config {
    fn from(args: &ArgMatches) -> Self {
        // The program takes a destination as input in the format username@host, typically something like
        // user@10.0.0.8 on a LAN. We parse this input as follows:
        // If the user input a hostname as an IP Address, we can just parse it as such - easy!
        // Otherwise, we're going to have to try to use DNS to resolve the hostname into an IP address.
        // If both of these options fail, we'll just have to yield an error message and close the program.
        let conn: Vec<&str> = args.value_of("DESTINATION").unwrap().split('@').collect();
        if conn.len() != 2 {
            eprintln!("Invalid destination format. Destination should be in the form `user@host`,");
            eprintln!("e.g. `someone@example.com` or `person@10.0.0.118`.");
            process::exit(1);
        }
        let user = String::from(conn[0]);
        let addr = if let Ok(ip) = conn[1].parse::<Ipv4Addr>() {
            ip.to_string()
        } else {
            lookup_host(conn[1])
                .unwrap_or_default()
                .get(1)
                .unwrap_or_else(|| {
                    eprintln!("Couldn't resolve remote server {} via DNS.", conn[1]);
                    process::exit(1);
                })
                .to_string()
        };

        // TODO: change this to a match statement to catch all possible arms?
        let auth_method = if args.is_present("password") {
            AuthMethod::Password(String::from(args.value_of("password").unwrap()))
        } else if args.is_present("privatekey") {
            AuthMethod::PrivateKey(String::from(args.value_of("privatekey").unwrap()))
        } else if args.is_present("manual") {
            AuthMethod::Manual
        } else {
            AuthMethod::Agent
        };

        // other config options - none of which will crash the program at this point
        let pubkey = match args.value_of("pubkey") {
            Some(path) => {
                let pk = Path::new(path);
                if pk.exists() {
                    Some(pk.to_owned())
                } else {
                    eprintln!("Public key not found.");
                    eprintln!("Attempting to authenticate with private key anyway.");
                    None
                }
            }
            None => None,
        };
        let passphrase = args.value_of("passphrase").map(String::from);
        let port: u16 = args.value_of("port").unwrap().parse().unwrap_or_else(|e| {
            eprintln!("Invalid port number: {e}");
            eprintln!("Using default port 22.");
            22
        });

        Self {
            user,
            addr,
            auth_method,
            pubkey,
            passphrase,
            port,
        }
    }
}

#[allow(unreachable_code, unused_variables, unused_mut)]
impl KeyboardInteractivePrompt for Config {
    fn prompt<'a>(
        &mut self,
        username: &str,
        instructions: &str,
        prompts: &[Prompt<'a>],
    ) -> Vec<String> {
        let mut responses: Vec<String> = Vec::with_capacity(prompts.len());

        Vec::new()
    }
}
