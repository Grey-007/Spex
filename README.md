<h1 align="center">𝙨𝙥𝙚𝙭</h1>
<p align="center"><strong>Dynamic wallpaper color palette generator and theming engine.</strong></p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-Stable-FF6B00?style=for-the-badge">
  <img alt="License" src="https://img.shields.io/badge/License-MIT-FFC300?style=for-the-badge">
  <img alt="Version" src="https://img.shields.io/badge/Version-0.1.0-00E5FF?style=for-the-badge">
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Linux-39FF14?style=for-the-badge">
  <img alt="CLI Tool" src="https://img.shields.io/badge/Type-CLI%20Tool-FF2D95?style=for-the-badge">
</p>

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
