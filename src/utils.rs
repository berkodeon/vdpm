use directories::BaseDirs;
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::{Result, VDPMError};

pub fn get_home_dir() -> PathBuf {
    if let Ok(vdpm_home) = std::env::var("VDPM_HOME") {
        return PathBuf::from(vdpm_home);
    }

    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.home_dir().to_path_buf()
    } else {
        panic!("Unable to find home directory")
    }
}

pub fn get_vd_version() -> Result<String> {
    let input = Command::new("vd")
        .args(["-v"])
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| VDPMError::CLICommandError("Cannot execute 'vd -v':", e))?;

    let output = String::from_utf8(input.stdout).map_err(|e| {
        VDPMError::StringFromUtf8Error("Couldn't convert stdout to utf-8 string: ", e)
    })?;

    let regex: Regex = Regex::new(r"(\d+.)?(\d+.)?(\*|\d+)").unwrap();

    match regex.find(&output) {
        Some(matching_result) => {
            let version = matching_result.as_str();

            if version.is_empty() {
                return Err(VDPMError::RegexMatchError(
                    "Version regex match result is empty",
                ));
            }

            Ok(version.to_owned())
        }
        _ => Err(VDPMError::RegexMatchError(
            "Couldn't extract version number from vd command output",
        )),
    }
}

pub fn hash<T: Hash>(value: &T) -> u64 {
    let mut s = DefaultHasher::new();
    value.hash(&mut s);
    s.finish()
}

