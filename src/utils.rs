use directories::{BaseDirs, ProjectDirs};
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::Result;
use anyhow::Context;

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

pub fn get_cache_dir() -> PathBuf {
    if let Ok(vdpm_home) = std::env::var("VDPM_HOME") {
        return PathBuf::from(vdpm_home).join(".cache").join("vdpm");
    }

    if let Some(proj_dirs) = ProjectDirs::from("", "", "vdpm") {
        proj_dirs.cache_dir().to_path_buf()
    } else {
        panic!("Unable to find cache directory")
    }
}

pub fn get_github_raw_base_url() -> String {
    std::env::var("VDPM_GITHUB_BASE_URL")
        .unwrap_or_else(|_| "https://raw.githubusercontent.com".to_string())
}

pub fn get_vd_version() -> Result<String> {
    let input = Command::new("vd")
        .args(["-v"])
        .stdout(Stdio::piped())
        .output()
        .context("failed to execute 'vd -v'")?;

    let output =
        String::from_utf8(input.stdout).context("'vd -v' output was not valid utf-8")?;

    let regex: Regex = Regex::new(r"(\d+.)?(\d+.)?(\*|\d+)").unwrap();

    match regex.find(&output) {
        Some(matching_result) => {
            let version = matching_result.as_str();

            if version.is_empty() {
                anyhow::bail!("'vd -v' output did not contain a parseable version");
            }

            Ok(version.to_owned())
        }
        _ => anyhow::bail!("could not find a version number in 'vd -v' output"),
    }
}

pub fn hash<T: Hash>(value: &T) -> u64 {
    let mut s = DefaultHasher::new();
    value.hash(&mut s);
    s.finish()
}

