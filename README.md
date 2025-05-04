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
