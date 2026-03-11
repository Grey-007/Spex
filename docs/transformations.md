# Spex Color Transformations

This document explains how to transform a palette color into different formats directly inside templates.

## Base Variable
Most transformations are built from a role or dynamic color name.

Example base variable:

```text
{{accent}}
```

If `accent` is `#FFAA33`, you can derive multiple variants from it.

## 1. RGB Transformation
Syntax:

```text
{{accent_rgb}}
```

Output format:

```text
R, G, B
```

Example:

```text
{{accent}} -> #FFAA33
{{accent_rgb}} -> 255, 170, 51
```

Useful when a target config expects comma-separated channel values.

## 2. RGBA Transformation
Syntax:

```text
{{accent_rgba(0.8)}}
```

Output format:

```text
rgba(R, G, B, A)
```

Example:

```text
{{accent}} -> #FFAA33
{{accent_rgba(0.5)}} -> rgba(255, 170, 51, 0.50)
```

Notes:
- Alpha is clamped to the range `0.0..=1.0`.
- This is useful for transparent overlays, borders, and shadows.

## 3. HSL Transformation
Syntax:

```text
{{accent_hsl}}
```

Output format:

```text
hsl(H, S%, L%)
```

Example:

```text
{{accent}} -> #FFAA33
{{accent_hsl}} -> hsl(36, 100%, 60%)
```

Useful for systems that prefer HSL color notation.

## 4. Lighten Transformation
Syntax:

```text
{{accent_lighten(10)}}
```

Behavior:
- Converts base color to HSL.
- Increases lightness by `10` percentage points.
- Converts back to hex.

Example:

```text
{{accent}} -> #FFAA33
{{accent_lighten(10)}} -> #FFBE66
```

Use this for hover states, highlights, or subtle emphasis.

## 5. Darken Transformation
Syntax:

```text
{{accent_darken(15)}}
```

Behavior:
- Converts base color to HSL.
- Decreases lightness by `15` percentage points.
- Converts back to hex.

Example:

```text
{{accent}} -> #FFAA33
{{accent_darken(15)}} -> #D98A1A
```

Use this for pressed states, borders, or darker contrast elements.

## Combining Variables and Transformations
You can use transformations with semantic roles and dynamic colors:

- `{{primary_rgb}}`
- `{{text_rgba(0.85)}}`
- `{{color3_hsl}}`
- `{{color5_lighten(8)}}`

## Error-Safe Behavior
If a variable or transformation cannot be resolved:
- rendering does not crash
- the placeholder is left unchanged in output

This makes template debugging easier for new users.
