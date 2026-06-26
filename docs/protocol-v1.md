# Node-IDE Protocol v1 (What + How)

Protocol version `1` — v3 ships the first two axes of the 5W1H declaration model.

## Axes (v3 scope)

| Axis | Meaning |
|------|---------|
| **What** | What payload flows across the port |
| **How** | How delivery is shaped (single message vs stream, etc.) |

When, Where, Who, and Why are reserved for v12.

## What presets

| ID | Meaning |
|----|---------|
| `any` | Wildcard — compatible with any other What preset |
| `text` | String/text payload (v2 default) |
| `json` | JSON structured payload |
| `bytes` | Raw bytes |
| `custom` | Escape hatch — **rejected in v3** |

## How presets

| ID | Meaning (v3) |
|----|----------------|
| `single` | One message per execution |
| `stream` | Multiple messages (declared only in v3) |
| `request-response` | Request/reply pattern (declared only) |
| `broadcast` | Fan-out delivery (declared only) |
| `custom` | Escape hatch — **rejected in v3** |

v3 runtime execution supports **single → single** only. Other How pairs are rejected at connect time.

## Resolution grid (v3)

For each axis, every preset pair is either **compatible** or **reject**:

- **What**: `any` matches all; identical presets match; otherwise reject.
- **How**: only `single` ↔ `single` is compatible; all other pairs reject.

Both axes must be compatible for a connection to succeed. Adapters (v13) are not inserted in v3 — incompatible pairs are always rejected with a human-readable reason and hint.

## Addition-only rule (D3)

Once a preset ID is released, its meaning must not change. New presets and axes may be added only. Existing graphs must keep working after updates via schema migration.

## Manifest example (node port)

```json
{
  "port_decls": {
    "out": {
      "what": { "preset": "text" },
      "how": { "preset": "single" }
    },
    "in": {
      "what": { "preset": "text" },
      "how": { "preset": "single" }
    }
  }
}
```

## node-sdk defaults (builtin)

Minimal nodes inherit `any` + `single` I/O unless overridden. Builtin kinds set explicit defaults:

- **Constant** — `out`: text·single
- **JsonConstant** — `out`: json·single
- **Echo** — `in`: text·single, `out`: text·single
