# skill/agents.md — the `omni agent` subcommand family

The `omni agent` group targets **agents** (the running process inside a pane) rather than the pane itself. It's the right entry point for orchestration that should survive layout changes.

## Target forms

Any `<target>` slot in `omni agent <subcommand>` accepts:

- the durable `term_<hex>` terminal id (survives public-pane-id compaction)
- a unique agent name set at `agent start`
- a detected agent label (e.g. `codex`, `claude`) when only one pane matches
- any pane id form that `omni pane` commands accept

## Starting agents

Spawn a named agent in its own new tab — single-pane tab, no shell + split sibling:

```bash
omni agent start reviewer --new-tab --no-focus -- codex
```

Spawn a named agent by splitting the current pane:

```bash
omni agent start worker --split right --no-focus -- claude
```

`--new-tab` is mutually exclusive with `--tab` and `--split`. With `--workspace <id>` the new tab lands in that workspace; without it the active workspace is used. With no existing workspaces, `--new-tab` creates a new workspace whose root pane is the agent.

`agent start <name>` enforces that `<name>` is unique session-wide (`agent_name_taken` error otherwise).

## Targeting an agent by name

```bash
omni agent send reviewer "review the fix in bug-report.md"
omni agent read reviewer --source recent --lines 200
omni agent wait reviewer --status idle --timeout 600000
omni agent focus reviewer
omni agent attach reviewer
```

`agent send` writes literal text only — follow up with `omni pane send-keys <target> Enter` if you need the keystroke. For a one-shot "text + Enter", use `omni pane run` instead.

## Listing and renaming

```bash
omni agent list                                  # all agents, all workspaces
omni agent get reviewer                          # info on one
omni agent rename reviewer code-reviewer         # change the name
omni agent rename reviewer --clear               # free the name, keep the process
```

`agent rename <target> --clear` removes the agent name but leaves the display label intact. After clearing, the agent is no longer addressable by its old name and the pane border may fall back to the detected agent label (e.g. `codex`).

## When to use `omni agent` vs `omni pane`

- `omni agent` for orchestration that should survive pane-id shifts: starting named agents, sending tasks, waiting for status, reading output.
- `omni pane` for layout-aware operations: splitting, closing, focusing by position, sending raw keys.

## Response shapes

| Command | Returns |
|---|---|
| `agent list`, `agent get`, `agent start`, `agent rename`, `agent focus` | JSON |
| `agent read` | Plain text (same as `pane read`) |
| `agent send`, `agent attach` | Nothing on success |
