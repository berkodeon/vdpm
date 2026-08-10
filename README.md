# 🧩 VisiData Plugin Manager (`vdpm`)

**`vdpm`** is a fast, extensible, and registry-aware plugin manager for [VisiData](https://www.visidata.org/), written in Rust.
It helps users **discover, install, manage, and explain** plugins across official and custom registries — with minimal effort and maximum control.

---

## ✨ Why?

VisiData’s plugin ecosystem is rich but underexplored.
`vdpm` brings clarity, structure, and convenience to plugin management with a **CLI-first experience** — tailored for data power users and plugin developers alike.

---

## 🔧 What Can It Do?

- 🔍 **Search** plugins by name, group, or tag
- 📦 **Install** plugins from official or custom registries (URL-based or local)
- 📁 **List** installed plugins with status and metadata
- 📴 **Disable/Enable** plugins without deleting them
- 📚 **Explain** plugins by parsing docstrings and README content
- 🌍 **Support multiple registries** (Docker-style config)
- 🔄 (Planned) **Auto-update**, version pinning, and deprecation tracking

---

## 💡 Designed For

- 🧑‍💻 **Plugin developers** who want a dev-friendly workflow (local sandbox support coming soon)
- 🗂️ **Data wranglers** who install and test many plugins
- ⚡ **CLI enthusiasts** who appreciate speed, clarity, and minimal dependencies

---

## 📦 Plugin Registries

`vdpm` supports static or dynamic registries:

```json
[
  {
    ???
  }
]
'''

---

## Testing

`tests/` contains a black-box scenario test suite (`rstest` + `assert_cmd` + `assert_fs` + `insta`) that drives the real compiled `vdpm` binary as a subprocess and asserts on its actual behavior. Ground rules for contributing tests here:

- **Arrange state only via real CLI calls** (`env.install(...)`, `env.enable(...)`, etc. on `VdpmTestEnv` in `tests/support/mod.rs`) — never hand-write files into the test's home directory. If a scenario needs a plugin installed, install it through the CLI, don't fabricate the `.py`/`.visidatarc` state directly.
- **Tests hit real GitHub on purpose** for the happy path (plugin downloads). Only the "install a nonexistent plugin" failure case is effectively mocked, and it's mocked for free — a deliberately-bogus plugin name gets a real 404 from GitHub, no mocking library involved.
- **Requires real VisiData installed**, pinned to the version in [`.visidata-version`](.visidata-version) (currently `3.1.1`) — CI reads the same file, so bumping the tested version is a one-line change in one place. Every `vdpm` invocation shells out to `vd -v` at startup, even for read-only commands like `list`.
- **`VDPM_HOME` is the isolation seam**, not `$HOME` — the harness points each test at its own temp directory via `VDPM_HOME`, which `get_home_dir()` checks before falling back to the real home dir. Never mutate the real `$HOME` from a test.
- Run with `cargo insta test`; review new/changed snapshots with `cargo insta review`.
