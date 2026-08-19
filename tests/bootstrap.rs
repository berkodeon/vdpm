#[allow(dead_code)]
mod support;

use assert_fs::prelude::*;
use support::known_plugins::STABLE_PLUGIN;
use support::{env, VdpmTestEnv};

#[rstest::rstest]
fn test_list_after_manual_visidatarc_deletion_recovers_enabled_state_from_registry(
    env: VdpmTestEnv,
) {
    // Given
    env.install(STABLE_PLUGIN).enable(STABLE_PLUGIN);
    env.assert_enabled(STABLE_PLUGIN);

    std::fs::remove_file(env.visidatarc_path().path())
        .expect("failed to delete .visidatarc for test setup");

    // When
    env.list().success();

    // Then
    env.assert_installed(STABLE_PLUGIN);
    env.assert_enabled(STABLE_PLUGIN);
}

#[rstest::rstest]
fn test_list_recovers_from_missing_registry_csv_by_bootstrapping_from_visidatarc(env: VdpmTestEnv) {
    // Given
    env.install(STABLE_PLUGIN).enable(STABLE_PLUGIN);
    env.plugins_csv_path().assert(predicates::path::exists());

    std::fs::remove_file(env.plugins_csv_path().path())
        .expect("failed to delete plugins.csv for test setup");

    // When
    env.list().success();

    // Then
    env.assert_enabled(STABLE_PLUGIN);
    env.plugins_csv_path().assert(predicates::path::exists());
}

#[rstest::rstest]
fn test_first_run_with_nonexistent_vdpm_home_creates_directory_tree() {
    // Given
    let tmp = assert_fs::TempDir::new().unwrap();
    let nested_home = tmp.path().join("does/not/exist/yet");
    assert!(!nested_home.exists());

    // When
    assert_cmd::Command::cargo_bin("vdpm")
        .unwrap()
        .args(["list"])
        .env("VDPM_HOME", &nested_home)
        .assert()
        .success();

    // Then
    assert!(nested_home.is_dir());
    assert_eq!(
        std::fs::read_to_string(nested_home.join(".visidatarc")).unwrap(),
        ""
    );
}
