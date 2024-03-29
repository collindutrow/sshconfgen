//! # SSID
//!
//! This module contains the function to get the currently connected SSID of the machine.

use std::{
    process::Command,
    str
};

/// Get the currently connected SSID
pub fn get_current_ssid() -> Result<String, &'static str> {
    if cfg!(target_os = "windows")
    {
        let output = Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
            .expect("Failed to execute command");

        let output_str = str::from_utf8(&output.stdout).unwrap();
        for line in output_str.lines() {
            if line.contains("SSID") && !line.contains("BSSID") {
                return Ok(line.split(":").nth(1).unwrap().trim().to_string());
            }
        }

        return Ok("".to_string());
    }
    else if cfg!(target_os = "linux")
    {
        let output = Command::new("iwgetid")
            .args(["-r"])
            .output()
            .expect("Failed to execute command");

        return Ok(str::from_utf8(&output.stdout).unwrap().trim().to_string());
    }
    else if cfg!(target_os = "macos") {
        let output = Command::new("networksetup")
            .args(["-getairportnetwork", "en0"])
            .output()
            .expect("Failed to execute command");

        let output_str = str::from_utf8(&output.stdout).unwrap();
        if let Some(start) = output_str.find(": ") {
            return Ok(output_str[start + 2..].trim().to_string());
        }

        return Ok("".to_string());
    }

    Err("Unsupported operating system")
}