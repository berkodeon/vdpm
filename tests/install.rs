#[allow(dead_code)]
mod support;

use support::known_plugins::STABLE_PLUGIN;
use support::{VdpmTestEnv, env};

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
        .failure()
        .stderr(predicates::str::contains("PluginError"));

    // Then
    env.assert_not_installed("definitely-not-a-real-plugin-xyz");
}
