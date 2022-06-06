use std::net::Ipv4Addr;
use dns_lookup::lookup_host;
use clap::{arg, Command, ArgMatches};

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");

pub fn args() -> ArgMatches {
    Command::new(PROGRAM_NAME)
        .author("benharmonics")
        .version("0.1.0")
        .about("Secure file transfer tool with graphical interface")
        .arg(arg!(<DESTINATION> "Required remote connection, e.g. user@hostip"))
        .arg(arg!(-i --identity "Input path to SSH identity file").number_of_values(1).conflicts_with("password"))
        .arg(arg!(-p --password "Input SSH password for remote server").number_of_values(1).conflicts_with("identity"))
        .arg(arg!(-f --fullscreen "Fullscreen mode (without help panel)").takes_value(false))
        .get_matches()
}

#[derive(Debug)]
pub enum AuthMethod {
    Password(String),
    Identity(String),
    Manual,
}

#[derive(Debug)]
/// Our `Config` struct keeps track of our SFTP destination user@addr,
/// as well as some other application configuration info.
pub struct Config {
    pub user: String,
    pub addr: String,
    pub fullscreen: bool,
    pub auth_method: AuthMethod,
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
        let hostname = String::from(conn[0]);
        let parsed_ip_res = conn[1].parse::<Ipv4Addr>();
        let addr = if parsed_ip_res.is_ok() {
            parsed_ip_res.unwrap().to_string()
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
        let auth_method = if args.is_present("password") {
            AuthMethod::Password(String::from(args.value_of("password").unwrap()))
        } else if args.is_present("identity") {
            AuthMethod::Identity(String::from(args.value_of("identity").unwrap()))
        } else {
            AuthMethod::Manual
        };

        Config { user: hostname, addr, fullscreen, auth_method }
    }
}