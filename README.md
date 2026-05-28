# omni

Terminal workspace manager for AI coding agents.

> omni is a fork of [herdr](https://github.com/ogulcancelik/herdr) by Ogulcan Celik, released under the same AGPL-3.0-or-later license. See [NOTICE](./NOTICE) for full attribution. The `omni-fork-point` git tag marks the upstream divergence point.

## Install

From source (requires Rust toolchain):

```bash
cargo install --path . --force
```

The binary lands at `~/.cargo/bin/omni`.

## Concepts

- **Workspace** — a project context (one repo, one folder). Contains tabs.
- **Tab** — a subcontext inside a workspace. Contains panes.
- **Pane** — a real terminal: shell, agent, server, log stream, anything.

Every pane has a stable `terminal_id` (`term_<hex>`) and a workspace-relative public id (`w<ws>-<n>`). Public ids compact when sibling panes close; `terminal_id` does not.

## Launch

```bash
omni                    # launch or attach to the default persistent session
omni --session <name>   # use a named session
omni server stop        # stop the running server
omni --help             # full command list
```

## Commands

| Group | What it does |
|---|---|
| `omni workspace ...` | list / create / focus / close workspaces |
| `omni tab ...`       | list / create / focus / rename / close tabs |
| `omni pane ...`      | list / get / split / read / send-text / send-keys / close / rename |
| `omni agent ...`     | start / list / send / read / wait / rename / attach |
| `omni wait ...`      | block on `output` match or `agent-status` change |

Pane targets accept the durable `term_<hex>` form or the public `w<ws>-<n>` form.
Agent targets additionally accept the unique agent name.

## Examples

### Observe a peer pane

```bash
omni pane list                                            # discover everyone
omni pane read w1-2 --source recent --lines 80            # read its scrollback
omni pane read term_652d... --source recent --lines 80    # same, durable id
```

### Round-trip task to another agent

```bash
omni pane run w1-2 "review the fix in bug-report.md"
omni wait agent-status w1-2 --status done --timeout 600000
omni pane read w1-2 --source recent --lines 200
```

### Spawn an agent in its own new tab (single-pane, no split)

```bash
omni agent start reviewer --new-tab --no-focus -- codex
```

### Spawn an agent by splitting the current tab

```bash
omni agent start worker --split right --no-focus -- claude
```

### Fan out and gather

```bash
P1=$(omni pane split $OMNI_PANE_ID --direction right --no-focus | jq -r .result.pane.pane_id)
P2=$(omni pane split "$P1" --direction down --no-focus | jq -r .result.pane.pane_id)
omni pane run "$P1" "cargo test"
omni pane run "$P2" "cargo clippy"
for p in "$P1" "$P2"; do
  omni wait agent-status "$p" --status done
  omni pane read "$p" --source recent --lines 40
done
```

## License

[AGPL-3.0-or-later](./LICENSE). See [NOTICE](./NOTICE) for attribution.
