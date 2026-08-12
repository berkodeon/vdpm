#[allow(dead_code)]
mod support;

use support::known_plugins::STABLE_PLUGIN;
use support::{AssertExt, VdpmTestEnv, env};

#[rstest::rstest]
fn test_enable_previously_installed_plugin(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN).enable(STABLE_PLUGIN);

    // Then
    env.assert_enabled(STABLE_PLUGIN);
    insta::assert_snapshot!(
        "enabling_plugin_adds_import_line_to_visidatarc",
        env.visidatarc_contents()
    );
}

#[rstest::rstest]
fn test_enable_never_installed_plugin_fails(env: VdpmTestEnv) {
    // When
    env.try_enable("not-installed")
        .assert_failure_containing("is not installed");

    // Then
    env.assert_disabled("not-installed");
}
