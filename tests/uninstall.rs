#[allow(dead_code)]
mod support;

use support::known_plugins::{ANOTHER_STABLE_PLUGIN, STABLE_PLUGIN};
use support::{VdpmTestEnv, env};

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
