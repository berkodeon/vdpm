#[allow(dead_code)]
mod support;

use support::known_plugins::{ANOTHER_STABLE_PLUGIN, STABLE_PLUGIN};
use support::{AssertExt, VdpmTestEnv, env};

#[rstest::rstest]
fn test_list_with_zero_plugins(env: VdpmTestEnv) {
    // Given a fresh env
    // When
    let output = env.list().stdout_string();

    // Then
    insta::assert_snapshot!("list_shows_empty_table_when_no_plugins_installed", output);
}

#[rstest::rstest]
fn test_list_with_mixed_enabled_disabled(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN)
        .install(ANOTHER_STABLE_PLUGIN)
        .enable(ANOTHER_STABLE_PLUGIN);

    let output = env.list().stdout_string();

    // Then
    insta::assert_snapshot!(
        "list_shows_each_plugin_with_its_own_installed_and_enabled_status",
        output
    );
}
