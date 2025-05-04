use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use directories::BaseDirs;

fn get_home_dir() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.home_dir().to_path_buf()
    } else {
        panic!("Unable to find home directory")
    }
}

fn create_plugin_directory(base_path: &PathBuf) -> PathBuf {
    let plugin_dir = base_path.join(".config").join("vdpm");

    if !plugin_dir.exists() {
        fs::create_dir_all(&plugin_dir).expect("Failed to create plugin directory");
    }

    plugin_dir
}

fn list_python_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap().filter_map(Result::ok) {
            let path = entry.path();
            if path.extension() == Some("py".as_ref()) {
                if let Some(file_name) = path.file_stem() {
                    if let Some(file_str) = file_name.to_str() {
                        files.push(file_str.to_string());
                    }
                }
            }
        }
    }
    files
}

fn write_to_file(file_path: &str, content: &Vec<String>) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    for line in content {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

fn main() {
    let home_dir = get_home_dir();
    let vdpm_plugin_dir = create_plugin_directory(&home_dir);

    let plugin_path = home_dir.join(".visidata/plugins");
    let python_files = list_python_files(&plugin_path);

    let vdpm_plugins_file = vdpm_plugin_dir.join("plugins.csv").to_str().unwrap().to_string();

    if let Err(e) = write_to_file(&vdpm_plugins_file, &python_files) {
        eprintln!("Error writing to file: {}", e);
        return;
    }

    let mut child = Command::new("vd")
        .arg(&vdpm_plugins_file)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start VisiData");

    println!("Visidata started with plugin list!");
    child.wait().expect("VisiData process failed");
    println!("Stopped VisiData");
}

