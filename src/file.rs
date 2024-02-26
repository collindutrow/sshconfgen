//! # File utilities
//!
//! This module contains file utilities for reading, writing, and manipulating files.

use regex::Regex;
use std::fs::read_dir;
use std::{fs::File, io, io::Read, io::Write, path::PathBuf};
use std::path::Path;
use crate::{is_verbose, verbose_println};

/// Append a borrowed string slice to a file
pub fn append_to_file(path: &PathBuf, contents: &str, append_newline: bool) -> io::Result<()> {
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
        let newline = if cfg!(windows) { "\r\n" } else { "\n" };
        if !contents.ends_with(newline) {
            verbose_println!("Appending newline to {}", path.display());
            contents.to_string().push_str(newline);
        }
    }

    let mut file = File::options().append(true).open(path)?;
    writeln!(file, "{}", contents)?;
    Ok(())
}

/// Get the contents between two strings
pub fn get_between(contents: &str, start: &str, end: &str) -> String {
    let re = Regex::new(&format!(
        "(?s){}(.*){}",
        regex::escape(start),
        regex::escape(end)
    ))
    .unwrap();
    if let Some(caps) = re.captures(contents) {
        caps.get(1)
            .map_or(String::new(), |m| m.as_str().trim().to_string())
    } else {
        String::new()
    }
}

/// Read the contents of a file
pub fn read_file(path: &PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Get all files in a directory with a specific extension
pub fn get_files_by_extension(directory: &Path, extension: &str) -> Vec<PathBuf> {
    read_dir(directory)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == extension) {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_else(|_| vec![])
}
