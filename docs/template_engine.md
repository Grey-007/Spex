# Spex Template Engine

## 1. Overview
The Spex template engine takes your generated palette and injects those colors into text templates.
This lets you automatically update config files for tools like Waybar, Rofi, terminals, editors, and launchers.

High-level flow:
1. Spex generates a palette from an image.
2. Spex assigns semantic roles (for example `background`, `accent`, `text`).
3. Spex loads template files.
4. Spex replaces template variables with palette values.
5. Spex writes the rendered output files.
6. Spex runs hooks (if configured and not using `--dry-run`).

The system is intentionally lightweight:
- no heavy template library
- simple string replacement
- safe behavior for missing variables

## 2. Directory Structure
Your Spex config directory should contain:

```text
~/.config/spex/
    config.toml
    templates/
```

- `config.toml`: defines what to render and where to write output.
- `templates/`: contains your template source files.

## 3. Example `config.toml`
Minimal example:

```toml
[[template]]
input = "~/.config/spex/templates/waybar.css"
output = "~/.config/waybar/colors.css"

[hooks]
commands = [
  "pkill -SIGUSR2 waybar"
]
```

Multiple template example:

```toml
[[template]]
input = "~/.config/spex/templates/waybar.css"
output = "~/.config/waybar/colors.css"

[[template]]
input = "~/.config/spex/templates/rofi.rasi"
output = "~/.config/rofi/colors.rasi"
```

## 4. Template Variable Usage
Spex supports semantic role variables:

- `{{background}}`
- `{{surface}}`
- `{{primary}}`
- `{{secondary}}`
- `{{accent}}`
- `{{accent2}}`
- `{{highlight}}`
- `{{text}}`

Example template snippet:

```css
window {
  background: {{background}};
  color: {{text}};
  border: 1px solid {{accent}};
}
```

Example rendered output:

```css
window {
  background: #0E1632;
  color: #E6F2FF;
  border: 1px solid #FFD166;
}
```

## 4.1 Stable Semantic Role Mapping (`colors.*`)
When using `colors.*` token paths, Spex keeps semantic role identity stable during rendering.

Direct mapping:
- `colors.background` -> `theme.background`
- `colors.surface` -> `theme.surface`
- `colors.primary` -> `theme.primary`
- `colors.secondary` -> `theme.secondary`
- `colors.accent` -> `theme.accent`
- `colors.accent2` -> `theme.accent2`
- `colors.highlight` -> `theme.highlight`
- `colors.text` -> `theme.text`

This means `{{colors.surface.hex}}` always resolves from the `surface` role, not an auto-substituted role.

Fallback behavior is intentionally limited and only used if a requested role is missing:
- `accent2` -> `accent`
- `surface` -> `background`
- `secondary` -> `primary`

Contrast safety:
- if a resolved role color is too close to `background` (`ΔE < 8`), Spex picks the closest palette color that restores separation

Debugging:
- run with `--debug-theme` to print both the final theme-role metrics and the resolution for each rendered `colors.*` token
- example:

```text
Template variable: colors.surface.hex
Resolved role: surface
Color: #2C4A54
```

## 4.2 Background and Surface Generation
Theme roles are selected in LAB space and then checked with Delta-E, saturation, and contrast rules.

Backgrounds are biased toward cleaner endpoints while keeping wallpaper tint:

- dark theme: pick the darkest palette color, then mix it 60% toward black
- light theme: pick the lightest palette color, then mix it 60% toward white

Surface selection stays close to the background without collapsing into it:

- `surface` -> the next palette color closest in theme-relative luminance, as long as `Delta-E(background, surface) > 8`
- `surface_container` -> a gentle step away from `surface`
- `surface_high` -> a second step away from `surface_container`

If a role gets too close to the background or would invert the theme order, Spex nudges lightness and saturation slightly to keep the ordering stable:

- background -> surface -> accents -> text

Accent role selection is saturation-first but still guarded by perceptual distance:

- `primary` -> highest-saturation candidate with `Delta-E(primary, background) > 20`
- `secondary` -> next saturated candidate with `Delta-E(secondary, primary) > 12` and `Delta-E(secondary, background) > 15`
- `accent` and `accent2` -> next saturated candidates that stay at least `Delta-E > 10` away from the already-selected roles
- `highlight` -> the unused color that maximizes separation from both `background` and `primary`
- `text` -> the highest-contrast endpoint relative to the background

If the palette does not have enough distinct colors, Spex applies small lightness and saturation nudges instead of inventing unrelated colors.

## 4.3 Dull Wallpaper Handling
If the extracted palette is low-saturation on average, Spex applies a restrained enhancement pass:

- average saturation below `0.25` triggers the boost
- saturation is increased slightly, typically in the 20-40% range
- contrast is increased slightly to keep surfaces and accents readable
- grayscale palettes can receive a small hue hint based on the wallpaper's dominant color region

The goal is to preserve the wallpaper's identity, not replace it with artificial colors.

Use `--debug-theme` to inspect the final role assignment pass. It prints each role's LAB values, luminance, saturation, contrast against background, and pairwise Delta-E distances.

## 5. Dynamic Palette Variables
You can also reference colors by index:

- `{{color0}}`
- `{{color1}}`
- `{{color2}}`
- ...

These map directly to the generated palette vector order.

Example:

```text
primary = {{color0}}
muted = {{color1}}
accent = {{color2}}
```

Palette size behavior:
- If you generate 8 colors, valid variables are typically `{{color0}}` to `{{color7}}`.
- If you generate 16 colors, valid variables are typically `{{color0}}` to `{{color15}}`.
- Out-of-range variables are left unchanged (they do not crash rendering).

## 6. Loop Syntax
Loop blocks iterate over the entire dynamic palette:

```text
{{#colors}}
color{{index}} = {{value}}
{{/colors}}
```

Inside a loop:
- `{{index}}` is the zero-based index (0, 1, 2, ...)
- `{{value}}` is the corresponding hex color (`#RRGGBB`)

Example output (shortened):

```text
color0 = #2E3440
color1 = #3B4252
color2 = #88C0D0
color3 = #A3BE8C
```

This is useful for tools that expect numbered color slots.

## 7. Multiple Template Directories
You can declare template search paths with `template_dirs.paths`:

```toml
[template_dirs]
paths = [
  "~/.config/spex/templates",
  "~/.local/share/spex/templates"
]
```

How Spex uses these paths:
1. It processes explicit `[[template]]` entries first.
2. It can discover additional template files from listed directories.
3. Duplicate template inputs are ignored to avoid double-rendering.

This allows sharing common template packs across systems.

## Tips for Beginners
- Start with one template file and confirm output.
- Use `--dry-run` to preview rendered content safely.
- Keep template files in version control when possible.
- Prefer semantic variables (`{{background}}`, `{{text}}`) for stable theming across different palettes.

## Template Validation and Diagnostics
Spex includes `spex doctor` to validate template health before applying changes.

Validation includes:
- checking that configured template files exist and are readable
- parsing template tokens
- validating dynamic color-engine tokens like `{{colors.surface.default.hex}}`

If an invalid token appears, Spex reports:
- file path
- line number
- offending token
- suggested role names

Example error:

```text
[ERROR] Invalid template token

File: ~/.config/spex/templates/vscode.json
Line: 42

Unknown token: surface_ultra_high

Did you mean:
surface_container_high
surface_container_highest
```
