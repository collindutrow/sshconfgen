use std::process::Command;
use std::str;

pub fn get_mac_address(ip_address: &str) -> Result<String, &'static str> {
    let command: &str;
    let args: Vec<&str>;

    #[cfg(target_os = "linux")] {
        command = "arp";
        args = vec!["-n", ip_address];
    }

    #[cfg(target_os = "macos")] {
        command = "arp";
        args = vec!["-n", ip_address];
    }

    #[cfg(target_os = "windows")] {
        command = "cmd";
        args = vec!["/C", &format!("arp -a {}", ip_address)];
    }

    if command.is_empty() {
        return Err("OS not supported");
    }

    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|_| "Failed to execute command")?
        .stdout;

    let mac_address = str::from_utf8(&output)
        .unwrap()
        .lines()
        .filter(|line| line.contains(ip_address))
        .flat_map(|line| line.split_whitespace().nth(if cfg!(target_os = "windows") { 1 } else { 2 }))
        .next()
        .ok_or("MAC address not found")?;

    Ok(mac_address.to_string())
}
