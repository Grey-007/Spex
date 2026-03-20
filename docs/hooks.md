# Spex Hooks

## 1. What Hooks Are
Hooks are shell commands that Spex runs after template rendering is complete.
They are useful for reloading applications that consume generated theme files.

Common examples:
- reloading Waybar
- refreshing compositor config
- restarting UI components that cache colors

Hooks are configured in `config.toml`:

```toml
[hooks]
commands = [
  "pkill -SIGUSR2 waybar",
  "hyprctl reload"
]
```

## 2. When Hooks Run
Hooks run only after all template rendering tasks finish successfully.

Execution timing:
1. generate palette
2. render templates
3. write output files
4. run hooks

Important behavior:
- If `--dry-run` is enabled, hooks do not run.
- If template rendering fails first, hooks are not executed.
- Hook commands run independently, so one failure does not stop later commands.

## 3. Multiple Hooks and Execution Order
When multiple hook commands are listed, Spex runs them in the same order as they appear in `commands`.

Example:

```toml
[hooks]
commands = [
  "pkill -SIGUSR2 waybar",
  "hyprctl reload"
]
```

Order:
1. `pkill -SIGUSR2 waybar`
2. `hyprctl reload`

This ordering matters when one command depends on the effects of a previous command.

Failure handling:
- if one hook exits with an error, Spex logs a warning
- remaining hooks still run
- hook failures do not abort the full hook phase

## Practical Example Flow
Given:
- one template writes `~/.config/waybar/colors.css`
- one template writes `~/.config/rofi/colors.rasi`
- hooks list contains Waybar reload and compositor reload

Spex run flow:
1. Palette is extracted from the source image.
2. Template files are rendered with fresh color values.
3. Output files are written to their configured locations.
4. Waybar reload command runs.
5. Compositor reload command runs.

Result:
- all rendered files are up to date before any process reloads.

## Tips for New Users
- Start with one hook command and verify it works manually in a shell first.
- Add more hooks gradually.
- Keep hooks idempotent where possible (safe to run repeatedly).
- Use `--dry-run` to validate template output without triggering hooks.

## Hook Validation in `spex doctor`
`spex doctor` validates hooks without executing them.

Checks performed:
- command string is not empty
- shell executable (`sh`) is available in `PATH`

Important:
- doctor mode never runs hook commands
- this keeps diagnostics safe and side-effect free
