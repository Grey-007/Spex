<p align="center">
  <img alt="spex banner" src="https://capsule-render.vercel.app/api?type=rounded&height=220&color=0:ff8fab,50:ffb703,100:7bdff2&text=spex&fontSize=78&fontColor=ffffff&fontAlignY=42&animation=fadeIn">
</p>

<h1 align="center">SPEX</h1>
<p align="center"><strong>Dynamic wallpaper color palette generator and theming engine.</strong></p>
<p align="center"><em>(pronounced: spex)</em></p>

<p align="center">
  <img alt="License" src="https://img.shields.io/badge/License-MIT-FF5D8F?style=for-the-badge&logo=opensourceinitiative&logoColor=white">
  <img alt="Crates" src="https://img.shields.io/badge/Crates.io-v0.1.0-00C2FF?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-Stable-FF8A00?style=for-the-badge&logo=rust&logoColor=white">
</p>
<p align="center">
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Linux-8DFF00?style=for-the-badge&logo=linux&logoColor=111111">
  <img alt="CLI Tool" src="https://img.shields.io/badge/Type-CLI%20Tool-FFB703?style=for-the-badge&logo=gnubash&logoColor=111111">
  <img alt="Templates" src="https://img.shields.io/badge/Templates-Enabled-7A5CFF?style=for-the-badge&logo=files&logoColor=white">
</p>

<p align="center">
  <a href="#installation">Installation</a> ·
  <a href="docs/cli.md">CLI</a> ·
  <a href="docs/template_engine.md">Template Engine</a> ·
  <a href="docs/transformations.md">Transforms</a>
</p>

<p align="center">〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️</p>

## Preview

`spex` generates terminal palette previews so you can quickly inspect extracted colors before exporting or rendering templates.

## Features

- 🎨 High-quality palette extraction from wallpapers
- 🌗 Dark and light theme modes
- 🧩 Config-driven template engine with loops and transformations
- ⚙️ Workflow automation with template hooks
- 🖥️ Terminal palette preview output
- 📦 Multiple export formats (`json`, `css`, `terminal`)
- 🧱 Modular and extensible Rust architecture

## Installation

### Install with Cargo

```bash
cargo install --path .
```

### Manual build

```bash
git clone <repo>
cd spex
cargo build --release
```

Add the binary to your PATH:

```bash
ln -s target/release/spex ~/.local/bin/spex
```

## Usage

Basic usage:

```bash
spex wallpaper.jpg
```

Custom palette size:

```bash
spex wallpaper.jpg --colors 16
```

Dark theme:

```bash
spex wallpaper.jpg --theme dark
```

Dry run:

```bash
spex wallpaper.jpg --dry-run
```

## Template System

Templates are configured through `config.toml` and rendered with generated palette variables.

Directory layout:

```text
~/.config/spex/
    config.toml
    templates/
```

Template variable examples:

```text
{{background}}
{{primary}}
{{accent}}
```

## CLI Examples

Preview only:

```bash
spex preview wallpaper.jpg
```

Generate shell completions:

```bash
spex completions fish
```

Diagnostics:

```bash
spex doctor
```

## Diagnostics

Use `spex doctor` to validate your environment before running full template workflows.

It checks:
- config file availability and parse validity
- template directories and template file readability
- template token validity (including `colors.*` token paths)
- hook command configuration (without executing hooks)
- Spex Color Engine token generation
- template rendering simulation with a mock palette

Example:

```bash
spex doctor
```

## Documentation

Detailed docs are available in:

- `docs/template_engine.md`
- `docs/transformations.md`
- `docs/hooks.md`
- `docs/cli.md`

## Contributing

Issues and pull requests are welcome. If you have ideas for improvements, new template targets, or workflow enhancements, feel free to open a discussion.

## Credits

Inspired by:

- [`pywal`](https://github.com/dylanaraps/pywal)
- [`matugen`](https://github.com/InioX/matugen)

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
