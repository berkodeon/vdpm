#[allow(dead_code)]
mod support;

use support::known_plugins::{ANOTHER_STABLE_PLUGIN, STABLE_PLUGIN};
use support::{AssertExt, VdpmTestEnv, env};

#[rstest::rstest]
fn test_uninstall_removes_file_and_disables(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN)
        .enable(STABLE_PLUGIN)
        .assert_enabled(STABLE_PLUGIN)
        .uninstall(STABLE_PLUGIN);

    // Then
    env.assert_not_installed(STABLE_PLUGIN)
        .assert_disabled(STABLE_PLUGIN);
}

#[rstest::rstest]
fn test_uninstall_leaves_other_plugins_untouched(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN)
        .install(ANOTHER_STABLE_PLUGIN)
        .enable(ANOTHER_STABLE_PLUGIN)
        .uninstall(STABLE_PLUGIN);

    // Then
    env.assert_not_installed(STABLE_PLUGIN)
        .assert_installed(ANOTHER_STABLE_PLUGIN)
        .assert_enabled(ANOTHER_STABLE_PLUGIN);
}

#[rstest::rstest]
fn test_uninstall_never_installed_plugin_fails(env: VdpmTestEnv) {
    // When
    env.try_uninstall("not-installed")
        .assert_failure_containing("is not installed");

    // Then
    env.assert_not_installed("not-installed");
}

#[rstest::rstest]
fn test_uninstall_never_enabled_plugin_succeeds(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN)
        .try_uninstall(STABLE_PLUGIN)
        .success();

    // Then
    env.assert_not_installed(STABLE_PLUGIN);
    env.assert_disabled(STABLE_PLUGIN);
}

#[rstest::rstest]
fn test_uninstall_all_plugins_leaves_clean_state(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN)
        .install(ANOTHER_STABLE_PLUGIN)
        .enable(STABLE_PLUGIN)
        .enable(ANOTHER_STABLE_PLUGIN)
        .uninstall(STABLE_PLUGIN)
        .uninstall(ANOTHER_STABLE_PLUGIN);

    // Then
    assert_eq!(env.visidatarc_contents(), "");
    let output = env.list().stdout_string();
    insta::assert_snapshot!("list_shows_empty_table_after_full_uninstall", output);
}
