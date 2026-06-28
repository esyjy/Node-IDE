# Node-IDE lifecycle v1

Lifecycle protocol version `1` — v4 ships persistent/ephemeral modes and an expanded state machine.

## States

| State | Meaning |
|-------|---------|
| `idle` | Resting, ready to run (ephemeral) or listening (persistent) |
| `initializing` | Brief transition during persistent `start` |
| `running` | Executing |
| `waiting` | Persistent node listening, no input yet (e.g. empty Echo) |
| `stopping` | Brief transition during persistent `stop` |
| `stopped` | Persistent node halted; requires `start` before run |
| `done` | Ephemeral run completed |
| `failed` | Execution error |

Legacy `created` deserializes as `idle` and is migrated to `idle` on schema v4.

## Modes

| Mode | Run completes to | Start/Stop |
|------|------------------|------------|
| `ephemeral` | `done` | N/A |
| `persistent` | `idle` | `start_node` / `stop_node` |

New nodes default to `ephemeral` + `idle`. Switching to `persistent` via `update_node_mode` sets lifecycle to `stopped` (unless `failed`).

## Transitions (summary)

**Ephemeral:** `idle|done|failed` → `running` → `done|failed`

**Persistent:** `stopped` → `initializing` → `idle|waiting`; `idle|waiting` → `running` → `idle|failed`; `idle|waiting|running` → `stopping` → `stopped`

## IPC

### Commands

- `start_node { id }` — persistent only
- `stop_node { id }` — persistent only
- `update_node_mode { request: { id, mode } }` — `ephemeral` | `persistent`
- `run_graph` — async; emits incremental lifecycle/output/delivery events with 120ms inter-node pacing

### Events

**`node:lifecycle`**

```json
{
  "node_id": "uuid",
  "lifecycle": "running",
  "previous": "idle",
  "lifecycle_mode": "ephemeral"
}
```

**`node:output`** — unchanged (`node_id`, `output`)

**`message:delivered`** — unchanged (`edge_id`, `envelope`)

## Schema v4

`NodeInstance` adds `lifecycle_mode: "ephemeral" | "persistent"` (default `ephemeral`).

Migration v3→v4: `created` → `idle`, adds `lifecycle_mode: ephemeral`.

## Instance ID

`NodeInstance.id` (UUID) is stable across `start`/`stop` and persistence. No separate runtime instance ID in v4.
