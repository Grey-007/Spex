# Spex CLI Guide

## 1. Overview
`spex` is a command-line tool for generating color palettes and theme files from an image.
It can:
- extract and preview a palette
- assign semantic theme roles
- export theme files
- render templates
- run post-render hooks

The CLI is powered by `clap`, so it includes structured help output, value validation, and shell completions.

## 2. Command Structure
Main form:

```bash
spex [OPTIONS] <IMAGE>
```

Subcommand form:

```bash
spex <SUBCOMMAND> [OPTIONS] [ARGS]
```

Supported subcommands:
- `generate`
- `preview`
- `daemon`
- `completions`
- `config`

## 3. Commands

### `generate`
Generate palette, assign roles, optionally export, then render templates and run hooks.

```bash
spex generate wallpaper.jpg
```

### `preview`
Generate and preview palette output only.
This mode does not run template rendering or hooks.

```bash
spex preview wallpaper.jpg
```

### `daemon`
Reserved command for daemon workflow.

```bash
spex daemon
```

### `completions`
Print shell completion script to stdout.

```bash
spex completions bash
spex completions zsh
spex completions fish
```

### `config`
Print resolved config path information (default and optional override).

```bash
spex config
spex config --config ~/.config/spex/config.toml
```

## 4. Global Options

### `--colors <N>`
Number of palette colors to generate.

- Default: `16`
- Example:
  ```bash
  spex wallpaper.jpg --colors 16
  ```

### `--theme <MODE>`
Theme role mode.

- Values: `dark`, `light`
- Example:
  ```bash
  spex wallpaper.jpg --theme light
  ```

### `--export <FORMAT>`
Export theme palette in a built-in format.

- Values: `json`, `css`, `terminal`
- Example:
  ```bash
  spex wallpaper.jpg --export css
  ```

### `--config <PATH>`
Override template config path (`config.toml`) used for rendering/hook execution.

```bash
spex wallpaper.jpg --config ~/.config/spex/config.toml
```

### `--dry-run`
Render templates and print outputs without writing files or running hooks.

```bash
spex wallpaper.jpg --dry-run
```

### `--verbose`
Print additional debug information (mode, dry-run state, export format, etc.).

```bash
spex wallpaper.jpg --verbose
```

### `--no-preview`
Disable terminal palette preview blocks.

```bash
spex wallpaper.jpg --no-preview
```

### `--version`
Show CLI version.

```bash
spex --version
```

## 5. Basic Usage Examples

### Basic usage
```bash
spex wallpaper.jpg
```

### Generate with 16 colors
```bash
spex wallpaper.jpg --colors 16
```

### Light theme + CSS export
```bash
spex wallpaper.jpg --theme light --export css
```

### Preview only
```bash
spex preview wallpaper.jpg --colors 12
```

### Generate without terminal preview
```bash
spex generate wallpaper.jpg --no-preview
```

## 6. Template Rendering Example
Render templates and run hooks from default config path:

```bash
spex generate wallpaper.jpg
```

Render using custom config file:

```bash
spex generate wallpaper.jpg --config ~/.config/spex/config.toml
```

Dry-run template rendering (safe preview mode):

```bash
spex generate wallpaper.jpg --dry-run --verbose
```

## 7. Shell Completion Setup

### Fish
```bash
spex completions fish > ~/.config/fish/completions/spex.fish
```

### Bash
```bash
spex completions bash > ~/.local/share/bash-completion/completions/spex
```

### Zsh
```bash
spex completions zsh > ~/.zfunc/_spex
```

Then ensure your shell loads that completion location (for example via shell startup config).

## 8. Example Workflows

### Workflow A: daily wallpaper theming
1. Run `spex generate ~/Pictures/wallpaper.jpg --export css`
2. Spex writes CSS palette.
3. Spex renders configured templates.
4. Spex runs hooks to reload UI components.

### Workflow B: test templates safely
1. Edit template files.
2. Run `spex wallpaper.jpg --dry-run --verbose`.
3. Inspect rendered output printed to terminal.
4. Re-run without `--dry-run` to apply.

### Workflow C: generate and inspect palette only
1. Run `spex preview wallpaper.jpg --colors 20`.
2. Review terminal palette blocks and semantic roles.
3. Tune options (`--theme`, `--colors`) before full generate mode.
