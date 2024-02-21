//! # SSH Config Generator
//!
//! This utility generates SSH client config based on user-defined rules.

use std::{
    io::{self},
    sync::atomic::{AtomicBool, Ordering}
};

mod ssid;
mod hwaddr;
mod ping;
mod help;
mod sshconf;
mod file;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CONFIG_EXTENSION: &str = "sshconf";

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// `println!` if the verbose flag is set
#[macro_export]
macro_rules! verbose_println {
    ($($arg:tt)*) => {
        if is_verbose() {
            println!($($arg)*);
        }
    };
}

fn main() -> io::Result<()> {
    for arg in std::env::args() {
        if arg == "-h" || arg == "--help" {
            help::print_help();
            std::process::exit(0);
        }

        if arg == "-v" || arg == "--verbose" {
            VERBOSE.store(true, Ordering::SeqCst);
        }

        if arg == "-V" || arg == "--version" {
            println!("{}", VERSION);
            std::process::exit(0);
        }

        // Check that .ssh directory exists and .ssh/conf.d directories exists
        let home_dir = match dirs::home_dir() {
            Some(path) => path,
            None => {
                eprintln!("Error: Unable to determine home directory");
                std::process::exit(1);
            }
        };

        let ssh_dir = home_dir.join(".ssh");
        let ssh_config_dir = ssh_dir.join("config.d/");

        if !ssh_dir.exists() {
            eprintln!("Error: .ssh directory does not exist");
            std::process::exit(1);
        }

        if !ssh_config_dir.exists() {
            eprintln!("Error: .ssh/conf.d directory does not exist");
            std::process::exit(1);
        }

        if arg.starts_with("--monitor-ssid") {
            sshconf::ssh_config_gen()?;

            // Check if the argument includes a duration
            if let Some(equals_pos) = arg.find('=') {
                // Extract the duration value after '='
                let duration_str = &arg[equals_pos + 1..];
                if let Ok(duration) = duration_str.parse::<u64>() {
                    // Convert duration to seconds and call monitor_ssid with duration
                    monitor_ssid(Some(duration))?;
                } else {
                    // Handle invalid duration value
                    eprintln!("Error: Invalid duration specified for --monitor-ssid.");
                    std::process::exit(1);
                }
            } else {
                // Call monitor_ssid without duration
                monitor_ssid(None)?;
            }

            std::process::exit(0);
        }
    }

    sshconf::ssh_config_gen()?;

    Ok(())
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

/// Parses the `.ssh/config.d/` directory at regular intervals and generates the SSH config file if
/// the SSID changes.
fn monitor_ssid(sleep_time: Option<u64>) -> io::Result<()> {
    let sleep_time = sleep_time.unwrap_or_else(|| 20);

    let mut current_ssid = ssid::get_current_ssid();
    verbose_println!("Current SSID: {}", current_ssid.clone().unwrap());

    // Loop forever, every 20 seconds.
    loop {
        verbose_println!("<<>>");
        std::thread::sleep(std::time::Duration::from_secs(sleep_time as u64));
        let new_ssid = ssid::get_current_ssid();
        if new_ssid != current_ssid {
            current_ssid = new_ssid;
            verbose_println!("New SSID: {}", current_ssid.clone().unwrap());
            sshconf::ssh_config_gen()?;
        }
    }
}