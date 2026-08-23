#[allow(dead_code)]
mod support;

use support::known_plugins::STABLE_PLUGIN;
use support::{env, AssertExt, VdpmTestEnv};

#[rstest::rstest]
fn test_install_adds_plugin_file(env: VdpmTestEnv) {
    // Given a fresh env
    // When
    let output = env.install(STABLE_PLUGIN).last_stdout();

    // Then
    env.assert_installed(STABLE_PLUGIN);
    insta::assert_snapshot!("installed_plugin_listed_as_installed_and_disabled", output);
}

#[rstest::rstest]
fn test_install_nonexistent_plugin_fails(env: VdpmTestEnv) {
    // When
    env.try_install("definitely-not-a-real-plugin-xyz")
        .assert_failure_containing("failed to download plugin");

    // Then
    env.assert_not_installed("definitely-not-a-real-plugin-xyz");
}

#[rstest::rstest]
fn test_install_already_installed_plugin_succeeds(env: VdpmTestEnv) {
    // Given
    env.install(STABLE_PLUGIN);
    let before = env.plugin_file_contents(STABLE_PLUGIN);

    // When
    env.try_install(STABLE_PLUGIN).success();

    // Then
    env.assert_installed(STABLE_PLUGIN);
    let after = env.plugin_file_contents(STABLE_PLUGIN);
    assert_eq!(before, after);
}

#[rstest::rstest]
fn test_install_failure_for_other_plugin_leaves_existing_install_untouched(env: VdpmTestEnv) {
    // Given
    env.install(STABLE_PLUGIN);
    let before = env.plugin_file_contents(STABLE_PLUGIN);

    // When
    env.try_install("definitely-not-a-real-plugin-xyz")
        .assert_failure_containing("failed to download plugin");

    // Then
    env.assert_installed(STABLE_PLUGIN);
    let after = env.plugin_file_contents(STABLE_PLUGIN);
    assert_eq!(before, after);
    env.assert_not_installed("definitely-not-a-real-plugin-xyz");
}

#[rstest::rstest]
fn test_install_fails_on_rate_limit(env: VdpmTestEnv) {
    env.stub_plugin_source("some-plugin", 429, "rate limit exceeded");

    env.try_install("some-plugin")
        .assert_failure_containing("failed to download plugin");
    env.assert_not_installed("some-plugin");
}

#[rstest::rstest]
fn test_install_fails_on_server_error(env: VdpmTestEnv) {
    env.stub_plugin_source("some-plugin", 500, "internal server error");

    env.try_install("some-plugin")
        .assert_failure_containing("failed to download plugin");
    env.assert_not_installed("some-plugin");
}
