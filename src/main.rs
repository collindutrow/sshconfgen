use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::PathBuf;
use regex::Regex;

mod ssid;
mod hwaddr;
mod ping;
mod help;

// Store the version number and make it accessible to the help module
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> io::Result<()> {
    // Check if -h or --help was passed, if so, print help and exit
    for arg in std::env::args() {
        if arg == "-h" || arg == "--help" {
            help::print_help();
            std::process::exit(0);
        }
    }

    // Check if -v or --version was passed, if so, print version and exit
    for arg in std::env::args() {
        if arg == "-v" || arg == "--version" {
            println!("{}", VERSION);
            std::process::exit(0);
        }
    }

    let current_ssid = match ssid::get_current_ssid() {
        Ok(ssid) => ssid,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // match get_mac_address(ip_address) {
    //     Ok(mac) => println!("MAC address for {}: {}", ip_address, mac),
    //     Err(e) => println!("Error: {}", e),
    // }

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
    let conf_file_extension = ".sshconf";

    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let sshd_config_backup_file = ssh_dir.join(&format!("config.{}.orig", timestamp));

    println!("Detected Network Name: {}", current_ssid);
    println!("SSH config backup created: {}", sshd_config_backup_file.display());

    if ssh_config_file.exists() {
        // Rename the file to a backup, it is not a directory
        fs::rename(&ssh_config_file, &sshd_config_backup_file)?;
    }

    let mut config_files = get_ssh_config_files(&ssh_config_dir, conf_file_extension)?;

    config_files.sort();

    for config_file in config_files {
        //println!("Processing {}", config_file);
        let config_file_path = ssh_config_dir.join(config_file);
        let config_file_contents = read_file(&config_file_path)?;

        let settings = get_between(&config_file_contents, "# CONFIG BEGIN", "# CONFIG END");
        let local_rules = get_between(&config_file_contents, "# LOCAL RULES BEGIN", "# LOCAL RULES END");
        let remote_rules = get_between(&config_file_contents, "# REMOTE RULES BEGIN", "# REMOTE RULES END");
        let global_rules = get_between(&config_file_contents, "# GLOBAL RULES BEGIN", "# GLOBAL RULES END");

        let mut use_local_config = false;

        for line in settings.lines() {
            let (key, value) = get_key_value(line);
            if key == "LocalSSID" {
                let value_array = value.split(',').collect();
                if get_ssid_match(&value_array, &current_ssid) {
                    use_local_config = true;
                    break;
                }
            }

            // A gateway is a remote host with a mac address like so "LocalGateway ip|mac,ip2|mac2,ip3|mac3"
            if key == "LocalGateway" {
                let value_array: Vec<&str> = value.split(',').collect();
                for gateway in value_array {
                    let gateway_array: Vec<_> = gateway.split('|').collect();
                    if gateway_array.len() == 2 {
                        let ip = gateway_array[0];
                        let mac = gateway_array[1];
                        if let Ok(mac_address) = hwaddr::get_mac_address(ip) {
                            if mac_address == mac {
                                println!("Using local ssh rules for gateway {} ({})", ip, mac);
                                use_local_config = true;
                                break;
                            }
                        }
                    }
                }
            }

            // A list of IP address to ping to determine if we are on a local network
            if key == "LocalPing" {
                let value_array: Vec<&str> = value.split(',').collect();
                for ip in value_array {
                    if ping::get_pingable(ip) {
                        println!("Using local ssh rules for pingable IP {}", ip);
                        use_local_config = true;
                        break;
                    }
                }
            }
        }

        if !global_rules.is_empty() {
            println!("Using global ssh rules from {}", config_file_path.display());
            append_to_file(&ssh_config_file, &global_rules, true)?;
        }

        if use_local_config {
            if !local_rules.is_empty() {
                println!("Using local ssh rules from {}", config_file_path.display());
                append_to_file(&ssh_config_file, &local_rules, true)?;
            }
        } else if !remote_rules.is_empty() {
            println!("Using remote ssh rules from {}", config_file_path.display());
            append_to_file(&ssh_config_file, &remote_rules, true)?;
        }
    }

    // Check if the config file was created, if not, restore the original.
    if !ssh_config_file.exists() {
        println!("Warning! New config doesn't exist. Restoring original SSH config file");
        fs::rename(&sshd_config_backup_file, &ssh_config_file)?;
    } else if ssh_config_file.exists() {
        let metadata = fs::metadata(&ssh_config_file)?;
        // if config is empty (file size), restore the original.
        if metadata.len() == 0 {
            println!("Warning! New config is empty. Restoring original SSH config file");
            fs::rename(&sshd_config_backup_file, &ssh_config_file)?;
        } else {
            // Assume the new config file is good, remove the backup.
            println!("New SSH config file created, removing backup.");
            fs::remove_file(&sshd_config_backup_file)?;
        }
    }

    Ok(())
}

fn get_ssh_config_files(dir: &PathBuf, extension: &str) -> io::Result<Vec<String>> {
    // Strip the leading dot from the extension if it exists
    let extension = extension.trim_start_matches('.');

    // New vector to hold the file names
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        // entry is the file. We need to unwrap it to get the file name
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                // Strip the leading dot from the extension if it exists
                let path_ext = ext.to_string_lossy().trim_start_matches('.').to_string();

                if path_ext == extension.to_string() {
                    files.push(path.file_name().unwrap().to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(files)
}

fn read_file(path: &PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn append_to_file(path: &PathBuf, contents: &str, append_newline: bool) -> io::Result<()> {
    // Check if the contents are empty, if so, return
    if contents.is_empty() {
        return Ok(());
    }

    // If the file doesn't exist, create it
    if !path.exists() {
        File::create(path)?;
    }

    // Check if the contents end with a newline, if not, append one
    if append_newline {
        // Account for windows, macOS, and linux.
        let newline = if cfg!(windows) { "\r\n" } else { "\n" };
        if !contents.ends_with(newline) {
            contents.to_string().push_str(newline);
        }
    }

    let mut file = File::options().append(true).open(path)?;
    writeln!(file, "{}", contents)?;
    Ok(())
}

fn get_key_value(line: &str) -> (String, String) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() == 2 {
        (parts[0].trim().to_string(), parts[1].trim().to_string())
    } else {
        (String::new(), String::new())
    }
}

fn get_between(contents: &str, start: &str, end: &str) -> String {
    let re = Regex::new(&format!("(?s){}(.*){}", regex::escape(start), regex::escape(end))).unwrap();
    if let Some(caps) = re.captures(contents) {
        caps.get(1).map_or(String::new(), |m| m.as_str().trim().to_string())
    } else {
        String::new()
    }
}

fn get_ssid_match(array: &Vec<&str>, current_ssid: &str) -> bool {
    array.iter().any(|&ssid| ssid == current_ssid)
}
