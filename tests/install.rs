#[allow(dead_code)]
mod support;

use assert_fs::prelude::*;
use support::known_plugins::{ANOTHER_STABLE_PLUGIN, STABLE_PLUGIN};
use support::{AssertExt, Source, VdpmTestEnv, env};

#[rstest::rstest]
fn test_install_adds_plugin_file(env: VdpmTestEnv) {
    // Given a fresh env
    // When
    let output = env.install(STABLE_PLUGIN, Source::Default).last_stdout();

    // Then
    env.assert_installed(STABLE_PLUGIN);
    support::redact_mock_port_settings().bind(|| {
        insta::assert_snapshot!(
            support::mode_suffixed("installed_plugin_listed_as_installed_and_disabled"),
            output
        );
    });
}

#[rstest::rstest]
fn test_install_nonexistent_plugin_fails(env: VdpmTestEnv) {
    // When
    env.try_install("definitely-not-a-real-plugin-xyz", Source::Default)
        .assert_failure_containing("failed to download plugin");

    // Then
    env.assert_not_installed("definitely-not-a-real-plugin-xyz");
}

#[rstest::rstest]
fn test_install_already_installed_plugin_succeeds(env: VdpmTestEnv) {
    // Given
    env.install(STABLE_PLUGIN, Source::Default);
    let before = env.plugin_file_contents(STABLE_PLUGIN);

    // When
    env.install(STABLE_PLUGIN, Source::Default);

    // Then
    env.assert_installed(STABLE_PLUGIN);
    let after = env.plugin_file_contents(STABLE_PLUGIN);
    assert_eq!(before, after);
}

#[rstest::rstest]
fn test_install_failure_for_other_plugin_leaves_existing_install_untouched(env: VdpmTestEnv) {
    // Given
    env.install(STABLE_PLUGIN, Source::Default);
    let before = env.plugin_file_contents(STABLE_PLUGIN);

    // When
    env.try_install("definitely-not-a-real-plugin-xyz", Source::Default)
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

    env.try_install("some-plugin", Source::Default)
        .assert_failure_containing("failed to download plugin");
    env.assert_not_installed("some-plugin");
}

#[rstest::rstest]
fn test_install_fails_on_server_error(env: VdpmTestEnv) {
    env.stub_plugin_source("some-plugin", 500, "internal server error");

    env.try_install("some-plugin", Source::Default)
        .assert_failure_containing("failed to download plugin");
    env.assert_not_installed("some-plugin");
}

#[rstest::rstest]
fn test_install_from_custom_url(env: VdpmTestEnv) {
    let vd_version = vdpm::utils::get_vd_version().unwrap();
    let source = format!(
        "https://raw.githubusercontent.com/saulpw/visidata/v{vd_version}/visidata/loaders/{ANOTHER_STABLE_PLUGIN}.py"
    );

    env.install("custom_plugin", Source::Custom(&source));

    env.assert_installed("custom_plugin");
    assert!(!env.plugin_file_contents("custom_plugin").is_empty());
}

#[rstest::rstest]
fn test_install_from_local_file(env: VdpmTestEnv) {
    let local_file = assert_fs::NamedTempFile::new("local_plugin_source.py").unwrap();
    local_file.write_str("# local file content\n").unwrap();
    let path = local_file.path().to_str().unwrap();

    env.install("local_plugin", Source::Custom(path));

    env.assert_installed("local_plugin");
    assert_eq!(
        env.plugin_file_contents("local_plugin"),
        "# local file content\n"
    );
}

#[rstest::rstest]
fn test_install_rejects_invalid_name(env: VdpmTestEnv) {
    env.try_install("../evil", Source::Default)
        .assert_failure_containing("invalid plugin name");
}

#[rstest::rstest]
fn test_install_from_source_rejects_invalid_name(env: VdpmTestEnv) {
    env.try_install("../evil", Source::Custom("https://example.com/whatever.py"))
        .assert_failure_containing("invalid plugin name");
}

#[test]
fn rewrites_github_blob_url_to_raw_url() {
    assert_eq!(
        vdpm::cli::commands::install::rewrite_github_blob_url(
            "https://github.com/someuser/somerepo/blob/main/plugins/foo.py"
        ),
        "https://raw.githubusercontent.com/someuser/somerepo/main/plugins/foo.py"
    );
}

#[test]
fn leaves_non_blob_urls_unchanged() {
    let raw_url = "https://raw.githubusercontent.com/someuser/somerepo/main/foo.py";
    assert_eq!(
        vdpm::cli::commands::install::rewrite_github_blob_url(raw_url),
        raw_url
    );
}

#[test]
fn leaves_urls_from_other_hosts_unchanged() {
    let url = "https://gitlab.com/someuser/somerepo/-/blob/main/foo.py";
    assert_eq!(vdpm::cli::commands::install::rewrite_github_blob_url(url), url);
}
