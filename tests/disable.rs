#[allow(dead_code)]
mod support;

use support::known_plugins::STABLE_PLUGIN;
use support::{AssertExt, VdpmTestEnv, env};

#[rstest::rstest]
fn test_disable_previously_enabled_plugin(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN)
        .enable(STABLE_PLUGIN)
        .assert_enabled(STABLE_PLUGIN)
        .disable(STABLE_PLUGIN);

    // Then
    env.assert_disabled(STABLE_PLUGIN);
    assert_eq!(env.visidatarc_contents(), "");
}

#[rstest::rstest]
fn test_disable_already_disabled_plugin_is_noop(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN)
        .enable(STABLE_PLUGIN)
        .disable(STABLE_PLUGIN)
        .try_disable(STABLE_PLUGIN)
        .success();

    // Then
    env.assert_disabled(STABLE_PLUGIN);
    assert_eq!(env.visidatarc_contents(), "");
}

#[rstest::rstest]
fn test_disable_never_installed_plugin_fails(env: VdpmTestEnv) {
    // When
    env.try_disable("not-installed")
        .assert_failure_containing("is not installed");

    // Then
    env.assert_disabled("not-installed");
}

#[rstest::rstest]
fn test_disable_never_enabled_plugin_succeeds(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN).try_disable(STABLE_PLUGIN).success();

    // Then
    env.assert_installed(STABLE_PLUGIN);
    env.assert_disabled(STABLE_PLUGIN);
}
