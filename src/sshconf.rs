//! # SSH Config Processor
//!
//! This module is responsible for processing config files and generating the new SSH config file.

use crate::file::get_files_by_extension;
use crate::{hwaddr, is_verbose, ping, ssid, verbose_println};
use std::{fs, io, path::PathBuf};

/// Generate a new SSH client config file.
pub fn ssh_config_gen() -> io::Result<()> {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: Unable to determine home directory");
            std::process::exit(1);
        }
    };

    let ssh_dir = home_dir.join(".ssh");
    let ssh_config_file = ssh_dir.join("config");
    let ssh_config_dir = ssh_dir.join("config.d/");
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let sshd_config_backup_file = ssh_dir.join(&format!("config.{}.orig", timestamp));

    parse_and_process(&ssh_config_dir, &ssh_config_file, &sshd_config_backup_file);
    cleanup(&ssh_config_file, &sshd_config_backup_file);

    Ok(())
}

/// Parse and process the config files.
fn parse_and_process(ssh_config_dir: &PathBuf, ssh_config_file: &PathBuf, sshd_config_backup_file: &PathBuf) {
    let mut config_files = get_files_by_extension(&ssh_config_dir, crate::CONFIG_EXTENSION);

    // If there are no config files, return early.
    if config_files.is_empty() {
        verbose_println!("No config files found in {}", ssh_config_dir.display());
        return;
    }

    config_files.sort();

    let mut new_ssh_config = String::new();

    for config_file in config_files {
        let config_file_path = ssh_config_dir.join(config_file);
        let config_file_contents = crate::file::read_file(&config_file_path).unwrap();

        if config_file_contents.is_empty() {
            verbose_println!(
                "Skipping empty or unreadable config file: {}",
                config_file_path.display()
            );
            continue;
        }

        let config_settings = crate::file::get_between(
            &config_file_contents,
            "# CONDITIONS BEGIN",
            "# CONDITIONS END",
        );

        let local_rules = crate::file::get_between(
            &config_file_contents,
            "# LOCAL CONFIG BEGIN",
            "# LOCAL CONFIG END",
        );

        let remote_rules = crate::file::get_between(
            &config_file_contents,
            "# REMOTE CONFIG BEGIN",
            "# REMOTE CONFIG END",
        );

        let global_rules = crate::file::get_between(
            &config_file_contents,
            "# GLOBAL CONFIG BEGIN",
            "# GLOBAL CONFIG END",
        );

        let use_local_config = local_rules_match(&config_file_path, config_settings);

        // New line delimiter for Windows or Unix
        let newline = if cfg!(windows) { "\r\n" } else { "\n" };

        if !global_rules.is_empty() {
            verbose_println!("Using global ssh rules from {}", config_file_path.display());
            new_ssh_config.push_str(&global_rules);
            new_ssh_config.push_str(newline);
        }

        if use_local_config {
            if !local_rules.is_empty() {
                // No need to verbose print here, the local matching functions already do that.
                new_ssh_config.push_str(&local_rules);
                new_ssh_config.push_str(newline);
            }
        } else if !remote_rules.is_empty() {
            verbose_println!("Using remote ssh rules from {}", config_file_path.display());
            new_ssh_config.push_str(&remote_rules);
        }
    }

    if !new_ssh_config.is_empty() {
        backup_config(&ssh_config_file, &sshd_config_backup_file);
        crate::file::append_to_file(&ssh_config_file, &new_ssh_config, true)
            .expect("Error, unable to append newline to .ssh/config");
    }
}

/// Cleanup the SSH config file and restore the original if necessary.
fn cleanup(ssh_config_file: &PathBuf, sshd_config_backup_file: &PathBuf) {
    // Check if the config file was created, if not, restore the original.
    if !ssh_config_file.exists() {
        verbose_println!("Warning! New config doesn't exist. Restoring original SSH config file");
        fs::rename(&sshd_config_backup_file, &ssh_config_file)
            .expect("Error, unable to restore original SSH config file.");
    } else if ssh_config_file.exists() {
        let metadata = fs::metadata(&ssh_config_file)
            .expect("Error, unable to get metadata for new SSH config file.");

        // if new config is empty (file size), restore the original.
        if metadata.len() == 0 {
            verbose_println!("Warning! New config is empty. Restoring original SSH config file");
            fs::rename(&sshd_config_backup_file, &ssh_config_file)
                .expect("Error, unable to restore original SSH config file.");
        } else {
            // Assume the new config file is good, remove the backup.
            verbose_println!("New SSH config file created, removing backup.");
            fs::remove_file(&sshd_config_backup_file)
                .expect("Error, unable to remove backup file.");
        }
    }
}

/// Backup the SSH config file.
fn backup_config(ssh_config_file: &PathBuf, sshd_config_backup_file: &PathBuf) {
    if ssh_config_file.exists() {
        verbose_println!(
            "SSH config backup created: {}",
            sshd_config_backup_file.display()
        );

        // Rename the file to a backup, it is not a directory
        fs::rename(&ssh_config_file, &sshd_config_backup_file)
            .expect("Error, unable to backup SSH config file.");
    }
}

/// Check if the LocalSSID, LocalGateway, or LocalPing keys are present and if any match.
fn local_rules_match(config_file_path: &PathBuf, config_settings: String) -> bool {
    let mut use_local_config: bool;

    for line in config_settings.lines() {
        let (key, value) = get_key_value(line);

        use_local_config = local_ssid_match(&config_file_path, &key, &value);

        if !use_local_config {
            use_local_config = local_gateway_match(&config_file_path, &key, &value);
        }

        if !use_local_config {
            use_local_config = local_ping_made(&config_file_path, &key, &value);
        }

        if use_local_config {
            return true;
        }
    }

    false
}

/// Check if the LocalSSID key is present and if the current SSID matches any of the SSIDs.
/// If the current SSID matches any of the SSIDs, return true.
fn local_ssid_match(config_file_path: &PathBuf, key: &String, value: &String) -> bool {
    if key != "LocalSSID" {
        return false;
    }

    let current_ssid = match ssid::get_current_ssid() {
        Ok(ssid) => ssid,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Create a value_array of SSIDs delimited by a comma, filter out any empty strings.
    let value_array: Vec<&str> = value.split(',').filter(|&x| !x.is_empty()).collect();

    // Check if the current SSID matches any of the SSIDs in the value_array.
    if value_array.iter().any(|&ssid| ssid == current_ssid) {
        verbose_println!(
            "Using local ssh rules for {} reason: ssid match {}",
            config_file_path.display(),
            current_ssid
        );

        return true;
    }

    false
}

/// Check if the LocalPing key is present and if any of the IP addresses are pingable.
/// If any of the IP addresses are pingable, return true.
fn local_ping_made(config_file_path: &PathBuf, key: &String, value: &String) -> bool {
    if key != "LocalPing" {
        return false;
    }

    // A list of IP address to ping to determine if we are on a local network
    let value_array: Vec<&str> = value.split(',').collect();
    for ip in value_array {
        if ping::get_pingable(ip) {
            verbose_println!(
                "Using local ssh rules for {} reason: ping success {}",
                config_file_path.display(),
                ip
            );

            return true;
        }
    }

    false
}

/// Check if the LocalGateway key is present and if the gateway matches an ip and hw address.
/// If the gateway matches an ip and hw address, return true.
fn local_gateway_match(config_file_path: &PathBuf, key: &String, value: &String) -> bool {
    if key != "LocalGateway" {
        return false;
    }

    // A gateway is a remote host with a hw address like so "LocalGateway ip|mac,ip2|mac2,ip3|mac3"
    let value_array: Vec<&str> = value.split(',').collect();
    for gateway in value_array {
        let gateway_array: Vec<_> = gateway.split('|').collect();
        if gateway_array.len() == 2 {
            let ip = gateway_array[0];
            let mac = gateway_array[1];
            if let Ok(mac_address) = hwaddr::get_hw_address(ip) {
                if mac_address == mac {
                    verbose_println!(
                        "Using local ssh rules for {} reason: gateway match {} ({})",
                        config_file_path.display(),
                        ip,
                        mac
                    );

                    return true;
                }
            }
        }
    }

    false
}

/// Get the key and value from a line of text.
/// The key and value are separated by a space.
pub fn get_key_value(line: &str) -> (String, String) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() == 2 {
        (parts[0].trim().to_string(), parts[1].trim().to_string())
    } else {
        (String::new(), String::new())
    }
}