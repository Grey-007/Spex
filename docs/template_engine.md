# Template Engine

## Variables
- `{{background}}`, `{{surface}}`, `{{primary}}`, `{{secondary}}`, `{{accent}}`, `{{accent2}}`, `{{highlight}}`, `{{text}}`
- Dynamic colors: `{{color0}}`, `{{color1}}`, ...

## Loops
Use:
```
{{#colors}}
color{{index}} = {{value}}
{{/colors}}
```

## Directory Structure
- `~/.config/spex/config.toml`
- `~/.config/spex/templates/`
- `~/.config/spex/docs/`

## Template Directories
Template files can be declared explicitly in `[[template]]` and discovered from `template_dirs.paths`.
