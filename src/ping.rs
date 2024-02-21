//! # Ping
//!
//! This module contains the function to ping a host.

use std::process::Command;

/// Get whether a host is pingable
pub fn get_pingable(host: &str) -> bool {
    let mut ping_output;

    // Loop up to 4 times until we get a successful ping.
    for _i in 0..2 {
        //println!("Pinging {} (attempt {})", host, _i + 1);
        if cfg!(target_os = "windows") {
            ping_output = Command::new("ping")
                .args([host, "-n", "1", "-w", "1000"])
                .output()
                .expect("Failed to execute command");
        } else {
            ping_output = Command::new("ping")
                .args([host, "-c", "1", "-W", "1"])
                .output()
                .expect("Failed to execute command");
        }

        if ping_output.status.success() {
            return true;
        }
    }

    false
}