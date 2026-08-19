use std::cell::RefCell;

use assert_fs::prelude::*;

pub mod known_plugins;

fn settings() -> &'static vdpm::config_loader::Settings {
    vdpm::config_loader::init(vdpm::config_loader::RuntimeSettings {
        vd_version: String::new(),
    });
    &vdpm::config_loader::load_or_create()
        .expect("failed to load embedded config.toml")
        .settings
}

pub struct VdpmTestEnv {
    home: Option<assert_fs::TempDir>,
    last_stdout: RefCell<String>,
}

impl VdpmTestEnv {
    pub fn new() -> Self {
        Self {
            home: Some(assert_fs::TempDir::new().unwrap()),
            last_stdout: RefCell::new(String::new()),
        }
    }

    fn home(&self) -> &assert_fs::TempDir {
        self.home.as_ref().unwrap()
    }

    pub fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        let assert = assert_cmd::Command::cargo_bin("vdpm")
            .unwrap()
            .args(args)
            .env("VDPM_HOME", self.home().path())
            .assert();
        *self.last_stdout.borrow_mut() = assert.stdout_string();
        assert
    }

    pub fn last_stdout(&self) -> String {
        self.last_stdout.borrow().clone()
    }

    pub fn visidatarc_contents(&self) -> String {
        std::fs::read_to_string(self.visidatarc_path().path()).unwrap_or_default()
    }

    fn plugin_file(&self, name: &str) -> assert_fs::fixture::ChildPath {
        self.home()
            .child(format!("{}/{name}.py", settings().plugin_folder))
    }

    pub fn plugin_file_contents(&self, name: &str) -> String {
        std::fs::read_to_string(self.plugin_file(name).path()).unwrap_or_default()
    }

    pub fn visidatarc_path(&self) -> assert_fs::fixture::ChildPath {
        self.home().child(&settings().rc_file)
    }

    pub fn plugins_csv_path(&self) -> assert_fs::fixture::ChildPath {
        let s = settings();
        self.home()
            .child(format!("{}/{}", s.vdpm_config_folder_path, s.plugin_manager_file))
    }

    pub fn install(&self, name: &str) -> &Self {
        self.try_install(name).success();
        self
    }
    pub fn enable(&self, name: &str) -> &Self {
        self.try_enable(name).success();
        self
    }
    pub fn disable(&self, name: &str) -> &Self {
        self.try_disable(name).success();
        self
    }
    pub fn uninstall(&self, name: &str) -> &Self {
        self.try_uninstall(name).success();
        self
    }

    pub fn try_install(&self, name: &str) -> assert_cmd::assert::Assert {
        self.run(&["install", name])
    }
    pub fn try_enable(&self, name: &str) -> assert_cmd::assert::Assert {
        self.run(&["enable", name])
    }
    pub fn try_disable(&self, name: &str) -> assert_cmd::assert::Assert {
        self.run(&["disable", name])
    }
    pub fn try_uninstall(&self, name: &str) -> assert_cmd::assert::Assert {
        self.run(&["uninstall", name])
    }

    pub fn list(&self) -> assert_cmd::assert::Assert {
        self.run(&["list"]).success()
    }

    pub fn assert_installed(&self, name: &str) -> &Self {
        self.plugin_file(name).assert(predicates::path::exists());
        self
    }
    pub fn assert_not_installed(&self, name: &str) -> &Self {
        self.plugin_file(name).assert(predicates::path::missing());
        self
    }
    pub fn assert_enabled(&self, name: &str) -> &Self {
        let contents = self.visidatarc_contents();
        let line = format!("import plugins.{name}");
        assert!(
            contents.lines().any(|l| l == line),
            "expected .visidatarc to contain plugin \"{name}\", got:\n{contents}"
        );
        self
    }
    pub fn assert_disabled(&self, name: &str) -> &Self {
        let contents = self.visidatarc_contents();
        let line = format!("import plugins.{name}");
        assert!(
            !contents.lines().any(|l| l == line),
            "expected .visidatarc NOT to contain plugin \"{name}\", got:\n{contents}"
        );
        self
    }
}

impl Default for VdpmTestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VdpmTestEnv {
    fn drop(&mut self) {
        if let Some(dir) = self.home.take() {
            if std::thread::panicking() {
                let path = dir.into_persistent().path().to_path_buf();
                eprintln!("test failed — VDPM_HOME preserved at {}", path.display());
            }
        }
    }
}

pub trait AssertExt {
    fn stdout_string(&self) -> String;
    fn assert_failure_containing(self, needle: &str) -> Self;
}

impl AssertExt for assert_cmd::assert::Assert {
    fn stdout_string(&self) -> String {
        String::from_utf8(self.get_output().stdout.clone()).unwrap()
    }

    fn assert_failure_containing(self, needle: &str) -> Self {
        self.failure().stderr(predicates::str::contains(needle))
    }
}

#[rstest::fixture]
pub fn env() -> VdpmTestEnv {
    VdpmTestEnv::new()
}
