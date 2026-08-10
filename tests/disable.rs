#[allow(dead_code)]
mod support;

use support::known_plugins::STABLE_PLUGIN;
use support::{VdpmTestEnv, env};

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
