<p align="center">
  <img alt="spex banner" src="assets/spexbanner.png">
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

## Theme Derivation

Spex derives theme roles from the extracted palette using luminance and saturation metrics:

- Luminance uses `0.2126*R + 0.7152*G + 0.0722*B`
- Saturation uses `(max(R,G,B) - min(R,G,B)) / max(R,G,B)`
- The palette is sorted by luminance before role selection
- Dark themes sort darkest-to-lightest, light themes sort lightest-to-darkest
- `background` is the first color in that sorted palette
- `surface` is the next color closest in luminance to `background`
- `primary`, `secondary`, `accent`, and `accent2` are assigned from the remaining colors in descending saturation order
- `text` is chosen from the remaining color with the highest contrast ratio against `background`
- Roles stay unique unless the palette is too small to provide enough distinct colors
- The terminal preview uses the same luminance ordering so preview output matches role selection

This keeps theme role assignment aligned with the extracted palette instead of relying on fixed palette indices.

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
