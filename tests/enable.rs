#[allow(dead_code)]
mod support;

use support::known_plugins::STABLE_PLUGIN;
use support::{AssertExt, Source, VdpmTestEnv, env};

#[rstest::rstest]
fn test_enable_previously_installed_plugin(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN, Source::Default)
        .enable(STABLE_PLUGIN);

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

#[rstest::rstest]
fn test_enable_already_enabled_plugin_is_noop(env: VdpmTestEnv) {
    // Given / When
    env.install(STABLE_PLUGIN, Source::Default)
        .enable(STABLE_PLUGIN)
        .enable(STABLE_PLUGIN);

    // Then
    env.assert_enabled(STABLE_PLUGIN);
    let expected_line = format!("import plugins.{STABLE_PLUGIN}");
    let contents = env.visidatarc_contents();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines, vec![expected_line.as_str()]);
}
