# spex-convert

## What `spex-convert` does
`spex-convert` converts templates from other theming systems into spex-compatible template syntax.

It helps migrate files from ecosystems like:
- pywal-style templates
- matugen-style templates
- CSS-style variable templates

Main goal:
- detect existing tokens
- classify semantic meaning
- map to spex tokens
- rewrite template with spex syntax

## CLI Usage

```bash
spex-convert <template_file>
```

Optional flags:

```bash
--analyze
--output <file>
--verbose
```

Examples:

```bash
spex-convert theme.css
spex-convert theme.css --analyze
spex-convert theme.css --output converted.css
```

## Supported Token Patterns
Extractor supports:
- `{token}`
- `{{token}}`
- `${token}`
- `$token`

Normalization examples:
- `{background}` -> `background`
- `{{colors.primary.default.hex}}` -> `colors.primary.default.hex`
- `$accent` -> `accent`

## Template System Detection
The converter uses lightweight heuristics.

Examples:
- tokens like `{color0}` + `{background}` -> pywal-style
- tokens like `{{colors.surface.default.hex}}` -> matugen-style
- tokens like `$background` -> CSS-variable style

## Semantic Classification
Known aliases are mapped into semantic categories/roles.

Examples:
- `background`, `bg`, `base_bg` -> `background`
- `foreground`, `fg`, `text`, `on_surface` -> `foreground`
- `primary`, `accent`, `main` -> `primary`
- `primary_bg` -> `primary_container`
- `danger`, `alert` -> `error`
- `surface_high` -> `surface_container_high`
- `on_surface_variant` -> `surface_variant`
- `color0..color255` and `palette0..palette255` -> `color0..color255`

## Fuzzy Matching
If token names are slightly different, fuzzy matching attempts a best-fit classification.

Methods:
- prefix checks
- substring checks
- Levenshtein distance threshold

Examples:
- `mainColor` -> primary
- `secondarycontainer` -> secondary_container
- `outline-alt` -> outline_variant

## Mapping to Spex Tokens
Mapped outputs include:
- `background` -> `{{colors.background.default.hex}}`
- `primary_container` -> `{{colors.primary_container.default.hex}}`
- `on_error` -> `{{colors.on_error.default.hex}}`
- `surface_container_high` -> `{{colors.surface_container_high.default.hex}}`
- `palette0` -> `{{color0}}`

## Analyze Mode
Use analyze mode for inspection without rewriting files:

```bash
spex-convert theme.css --analyze
```

Analyze mode:
1. extracts tokens
2. classifies semantics
3. prints mapping suggestions
4. does not write any files

Example output:

```text
Detected tokens:

background
on_primary
surface_high
color0
accent

Suggested mappings:

background -> colors.background.default.hex
on_primary -> colors.on_primary.default.hex
surface_high -> colors.surface_container_high.default.hex
color0 -> color0
accent -> colors.primary.default.hex
```

## Converting a pywal template
Input:

```text
background: {background}
color: {foreground}
```

Output:

```text
background: {{colors.background.default.hex}}
color: {{colors.foreground.default.hex}}
```

## Output Behavior
Default output file:

```text
converted_template.spex
```

Custom output path:

```bash
spex-convert theme.css --output converted.css
```

## Error Handling
`spex-convert` reports:
- unreadable or missing file paths
- regex extraction/syntax problems
- unknown tokens (reported, conversion continues)

The tool stays lightweight and uses regex + semantic mapping for compatibility with spex templates.
