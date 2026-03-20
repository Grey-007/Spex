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
  <a href="docs/extraction.md">Extraction</a> ·
  <a href="#spex-convert">spex-convert</a> ·
  <a href="docs/template_engine.md">Template Engine</a> ·
  <a href="docs/transformations.md">Transforms</a>
</p>

<p align="center">〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️〰️</p>

<p>
  <img alt="Development Warning" src="https://img.shields.io/badge/Warning-Under%20Active%20Development-ff0000?style=for-the-badge&labelColor=000000">
  <img alt="OnHold" src="https://img.shields.io/badge/OnHold-cuz%20of%20Studies-E3C36B?style=for-the-badge&labelColor=000000">
   <img alt="Contribution-Welcome" src="https://img.shields.io/badge/Contributions-Welcome-brightgreen?style=for-the-badge&labelColor=000000"

</p>

> [!WARNING]
> Spex is still under active development and may behave inconsistently with some wallpapers.
> Palette generation can occasionally produce incorrect colors while the extraction and role-mapping logic is still being improved.

## Preview

`spex` generates terminal palette previews so you can quickly inspect extracted colors before exporting or rendering templates.


## Features

- 🎨 High-quality palette extraction from wallpapers
- 🧠 LAB k-means extraction with mediancut fallback
- 🌗 Dark and light theme modes
- 🧩 Config-driven template engine with loops and transformations
- ⚙️ Workflow automation with template hooks
- 🖥️ Terminal palette preview output
- 📦 Multiple export formats (`json`, `css`, `terminal`)
- 🔁 Companion `spex-convert` CLI for migrating external templates
- 🧱 Modular and extensible Rust architecture

## Installation

### Install with Cargo

```bash
cargo install --path .
```

This installs both CLI binaries:

- `spex`
- `spex-convert`

### Manual build

```bash
git clone <repo>
cd spex
cargo build --release
```

Add the binaries to your PATH:

```bash
ln -s target/release/spex ~/.local/bin/spex
ln -s target/release/spex-convert ~/.local/bin/spex-convert
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

Force mediancut:

```bash
spex wallpaper.jpg --extractor mediancut
```

Inspect extraction:

```bash
spex wallpaper.jpg --debug-extractor --debug-palette
```

## spex-convert

`spex-convert` is the companion migration tool in this repo. It converts templates from other theming systems into Spex-compatible placeholders so existing configs are easier to move into a Spex workflow.

What it does:

- extracts tokens from template files
- detects common source styles such as pywal, Matugen, and CSS-like variables
- classifies semantic aliases with direct and fuzzy matching
- rewrites recognized tokens into Spex syntax
- reports unknown tokens while leaving unmatched content untouched

It understands token forms such as `{token}`, `{{token}}`, `${token}`, `$token`, and nested Matugen braces like `{{{{colors.primary.default.hex}}}}`, which are normalized into canonical Spex tokens like `{{colors.primary.hex}}`.

Examples:

```bash
spex-convert theme.css
spex-convert theme.css --analyze
spex-convert theme.css --output converted.css
```

Default behavior writes the converted template to `converted_template.spex`. `--analyze` prints detected tokens and suggested mappings without writing a file.

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
{{colors.surface.hex}}
```

`colors.*` semantic role tokens are stable and map directly to generated theme roles:

- `colors.background` -> `theme.background`
- `colors.surface` -> `theme.surface`
- `colors.primary` -> `theme.primary`
- `colors.secondary` -> `theme.secondary`
- `colors.accent` -> `theme.accent`
- `colors.accent2` -> `theme.accent2`
- `colors.highlight` -> `theme.highlight`
- `colors.text` -> `theme.text`

Role fallback is only used when a requested role is missing:
- `accent2` -> `accent`
- `surface` -> `background`
- `secondary` -> `primary`

Theme background behavior:
- dark mode picks the darkest palette color, then mixes it 70% toward black for a deeper near-black background that still keeps the wallpaper tint
- light mode picks the lightest palette color, then mixes it 70% toward white for a cleaner near-white background with a slight wallpaper tint
- layered surfaces are derived from that background: `surface` (+8%), `surface_container` (+12%), `surface_high` (+18%)

Low-saturation wallpaper handling:
- if average palette saturation is below `0.25`, Spex applies a restrained vibrancy pass
- the boost raises saturation and contrast slightly instead of replacing the palette
- grayscale palettes can receive a small hue hint from the wallpaper's dominant color region

For troubleshooting template role resolution, run with:

```bash
spex generate wallpaper.jpg --debug-theme
```

For palette metrics and final role diagnostics, run with:

```bash
spex generate wallpaper.jpg --debug-colors
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

- `docs/extraction.md`
- `docs/template_engine.md`
- `docs/transformations.md`
- `docs/hooks.md`
- `docs/cli.md`
- `docs/spex-convert.md`

## Contributing

Issues and pull requests are welcome. If you have ideas for improvements, new template targets, or workflow enhancements, feel free to open a discussion.

## Credits

Inspired by:

- [`pywal`](https://github.com/dylanaraps/pywal)
- [`matugen`](https://github.com/InioX/matugen)

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
