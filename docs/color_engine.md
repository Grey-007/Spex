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

Example keys:
- `background`
- `surface`
- `primary`
- `surface_container_high`
- `outline_variant`

## Core Semantic Roles
Current role set includes:
- `background`
- `surface`
- `primary`
- `secondary`
- `tertiary`
- `outline`
- `outline_variant`
- `border`
- `surface_container_low`
- `surface_container`
- `surface_container_high`
- `surface_container_highest`
- `highlight`
- `selection`

## Derived Role Generation
SCE uses deterministic color derivation utilities:
- `lighten(color, percent)`
- `darken(color, percent)`
- `desaturate(color, factor)`
- `rotate_hue(color, degrees)`

These are used to build derived roles such as:
- container layers
- outlines
- tertiary variants

## Template Token Paths
Templates can request formatted token paths such as:

```text
{{colors.primary.default.hex}}
{{colors.primary.default.rgb}}
{{colors.primary.default.rgba(0.8)}}
{{colors.primary.default.hsl}}
```

Supported formats:
- `hex`
- `rgb`
- `rgba(alpha)`
- `hsl`

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
- checking required roles exist (`background`, `surface`, `primary`, `secondary`)
- validating token paths referenced in templates

This helps catch token naming issues early in config and template workflows.
