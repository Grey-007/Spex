# Spex Color Engine (SCE)

## Overview
The Spex Color Engine converts a raw palette into semantic tokens that templates can consume.
It is designed to be expandable and map-driven.

Core idea:
- tokens are stored dynamically in `HashMap<String, Color>`
- roles are not tied to a rigid fixed struct
- new roles can be added by inserting new map entries

## Token Model

```rust
pub struct SpexColorTokens {
    pub colors: HashMap<String, Color>
}
```

## Semantic Role Families
SCE ships with a wide semantic set for compatibility with modern systems (Matugen, VSCode-like themes).

Primary family:
- `primary`
- `primary_container`
- `on_primary`
- `on_primary_container`

Secondary family:
- `secondary`
- `secondary_container`
- `on_secondary`
- `on_secondary_container`

Tertiary family:
- `tertiary`
- `tertiary_container`
- `on_tertiary`
- `on_tertiary_container`

Error family:
- `error`
- `error_container`
- `on_error`
- `on_error_container`

Surface system:
- `surface`
- `surface_variant`
- `surface_container`
- `surface_container_low`
- `surface_container_high`
- `surface_container_highest`

UI tokens:
- `outline`
- `outline_variant`
- `border`
- `highlight`
- `selection`

Compatibility base roles:
- `background`
- `foreground`

## Derived Role Generation
SCE uses deterministic color derivation utilities:
- `lighten(color, percent)`
- `darken(color, percent)`
- `desaturate(color, factor)`
- `rotate_hue(color, degrees)`

These are used to derive containers, `on_*` contrast tokens, surface layers, and UI edge states.

## Template Token Paths
Templates can request formatted token paths such as:

```text
{{colors.primary.default.hex}}
{{colors.primary.default.rgb}}
{{colors.primary.default.rgba}}
{{colors.primary.default.rgba(0.8)}}
{{colors.primary.default.hsl}}
```

Supported formats:
- `hex`
- `rgb`
- `rgba`
- `rgba(alpha)`
- `hsl`

## Converter Compatibility
`/src/convert` maps common aliases (`accent`, `danger`, `on_surface_variant`, `surface_high`, etc.) into these semantic roles.
This reduces unknown token output when converting Matugen/CSS/VSCode-style templates.

## Expandability
Adding new semantic roles is simple:

```rust
tokens.colors.insert("primary_variant".to_string(), color);
tokens.colors.insert("status_error".to_string(), color);
tokens.colors.insert("accent_soft".to_string(), color);
```

No template-engine architecture changes are required for new role keys.

## Diagnostics Integration
`spex doctor` validates SCE by:
- generating tokens from a mock palette
- checking required role families exist in dark/light/inferred generation
- validating token paths referenced in templates

This helps catch token naming issues early in config and template workflows.
