use tokio::fs::OpenOptions;

use crate::error::{Result, VDPMError};
use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn list_files_by_extension(dir: &Path, extension: String) -> HashSet<String> {
    let mut files = HashSet::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)
            .unwrap()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.extension() == Some(extension.as_ref()) {
                if let Some(file_name) = path.file_stem() {
                    if let Some(file_str) = file_name.to_str() {
                        files.insert(file_str.to_string());
                    }
                }
            }
        }
    }
    files
}

pub async fn create_visidata_rc(rc_file_path: &Path) -> Result<()> {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(rc_file_path)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(VDPMError::VisidataRCError(
            "Failed to create .visidatarc".into(),
            e,
        )),
    }
}
