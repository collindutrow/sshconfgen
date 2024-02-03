use std::process::Command;
use std::str;

pub fn get_mac_address(ip_address: &str) -> Result<String, &'static str> {
    #[cfg(target_os = "linux")]
        let command = "arp";

    #[cfg(target_os = "macos")]
        let command = "arp";

    #[cfg(target_os = "windows")]
        let command = "cmd";

    #[cfg(any(target_os = "linux", target_os = "macos"))]
        let args = vec!["-n", ip_address];

    #[cfg(target_os = "windows")]
        let args = vec!["/C", &format!("arp -a {}", ip_address)];

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
