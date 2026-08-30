# vdpm

A small CLI for managing [VisiData](https://www.visidata.org/) plugins — install them, toggle them on and off, and see what's currently active, without hand-editing `.visidatarc`.

VisiData plugins are just Python files you drop into a folder and `import` from your `.visidatarc`. That's fine when you have one or two, but it gets tedious fast: you're manually downloading files, remembering to add/remove import lines, and keeping track of what's actually installed versus enabled. `vdpm` does that bookkeeping for you.

## Features

- **Install** a plugin by name — `vdpm` fetches it from VisiData's own [plugin loaders](https://github.com/saulpw/visidata/tree/develop/visidata/loaders), pinned to whatever version of `vd` you have installed, so you always get a compatible copy.
- **Install from anywhere else** — pass `--source` with a GitHub URL (blob links are rewritten to raw automatically), any other HTTP(S) URL, or a local file path, for plugins that aren't in the official repo.
- **Enable / disable** without uninstalling — flips the `import` line in `.visidatarc` on or off, leaving the file on disk untouched.
- **Uninstall** — disables and deletes the plugin file in one step.
- **List** everything you have installed, with its enabled/disabled state, in a clean table.
- **Interactive mode** (`vdpm interactive`) — opens your plugin registry as a spreadsheet *inside VisiData itself*. Toggle a plugin's `enabled` column, save, and `vdpm` watches the file, diffs what changed, and applies it for real (enabling/disabling/installing/uninstalling as needed). Managing your plugins by editing a table of plugins, in the tool the plugins are for, is the whole point.
- **Self-healing registry** — the list of installed plugins (`plugins.csv`) is regenerated from the actual plugin folder and `.visidatarc` every time you run a command, so a deleted or corrupted registry file just gets rebuilt instead of breaking things.
- **Validated plugin names** — names are restricted to letters, digits, `_`, and `-` at every entry point (CLI args and the registry file alike), so a stray or hand-edited row in `plugins.csv` can't smuggle something unexpected into `.visidatarc`.

## Requirements

- [VisiData](https://www.visidata.org/) installed and on your `PATH` as `vd` — `vdpm` shells out to `vd -v` on every run to detect your version, and `interactive` mode launches `vd` directly.
- A Rust toolchain supporting the 2024 edition (rustc 1.85+) if building from source.

## Installing

```sh
git clone https://github.com/berkodeon/vdpm.git
cd vdpm
cargo install --path .
```

This installs the `vdpm` binary to your cargo bin directory.

## Usage

```sh
vdpm list                          # show installed plugins and their status
vdpm install toml                  # install the official "toml" plugin
vdpm install my_plugin --source https://github.com/me/my_plugin/blob/main/my_plugin.py
vdpm install local_plugin --source ./my_plugin.py
vdpm enable toml                   # activate it in .visidatarc
vdpm disable toml                  # deactivate it without deleting the file
vdpm uninstall toml                # disable + remove the plugin file
vdpm interactive                   # edit your plugins as a spreadsheet in VisiData
```

By default, plugin files live in `~/.visidata/plugins`, the registry is `~/.config/vdpm/plugins.csv`, and activation happens through `~/.visidatarc` — all configurable in `config.toml`.

Two environment variables are recognized: `VDPM_HOME` overrides the home directory `vdpm` operates in (handy for sandboxing or testing), and `VDPM_GITHUB_BASE_URL` overrides the GitHub raw-content host used for the default plugin source.

## Developer guide

### Build

```sh
cargo build
cargo run -- list
```

### Run the tests

```sh
cargo test
```

Tests are black-box: they run the real compiled `vdpm` binary against a temp `VDPM_HOME` (via `assert_cmd`/`assert_fs`) and assert on its actual behavior — file contents, `.visidatarc` state, exit codes. By default, plugin downloads are served by a local mock server (`httpmock`), so the suite needs no network access. To instead run against real GitHub:

```sh
VDPM_TEST_GITHUB_MODE=real cargo test
```

Either way, you'll need `vd` installed and on `PATH`, pinned to the version in [`.visidata-version`](.visidata-version) — every `vdpm` invocation checks it at startup, even for read-only commands.

Several tests use [`insta`](https://insta.rs/) for snapshot assertions. Install the companion tool once:

```sh
cargo install cargo-insta --locked
cargo insta test      # run tests, capture new/changed snapshots for review
cargo insta review    # accept or reject pending snapshot changes
```

CI (`.github/workflows/rust.yml`) installs the pinned VisiData version, builds, and runs `cargo insta test --unreferenced=reject` on every PR.

A few ground rules for contributing tests, enforced by convention rather than the compiler:

- Arrange state through the CLI (`env.install(...)`, `env.enable(...)`, etc., via `VdpmTestEnv` in `tests/support/mod.rs`), not by hand-writing files into the test's home directory — unless the test is specifically about recovering from a corrupted/hand-edited registry, in which case that's the point.
- Never touch the real `$HOME`. `VDPM_HOME` is the isolation seam — `get_home_dir()` checks it before falling back to the real home directory, and `VdpmTestEnv` sets it to a fresh temp dir per test.

## License

Apache 2.0 — see [LICENSE](LICENSE).
