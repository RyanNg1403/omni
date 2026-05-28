# references/waits.md — deeper wait patterns

This file complements the wait section in the hub `SKILL.md`. The hub covers the common cases (`wait output`, `wait agent-status`, timeouts, the `recent-unwrapped` matching warning). Read this when you need stronger coordination guarantees or are debugging why a wait isn't firing.

## How matching works internally

Both `omni wait output` and `omni wait agent-status` resolve their target once at probe time and store the resolved `terminal_id`. From that point on:

- **`wait agent-status`** compares incoming `pane.agent_status_changed` events by `terminal_id`, not by public `pane_id`. So if a lower-numbered sibling pane closes mid-wait and the target's public id compacts, the wait keeps observing the same agent.
- **`wait output`** polls `pane read` against the resolved `terminal_id`. Same durability property — the wait does not lose its target across compaction.

This is why passing a `term_<hex>` is the safest target form for any long-running wait.

## `--source` semantics on `wait output`

`wait output` matches against `recent-unwrapped` text by default — soft-wrapped lines are joined before the matcher sees them, so pane width never breaks a match. The `--source` flag still exists for parity with `pane read`, but you'll almost always want the default.

If a wait isn't firing and you expect it to, inspect the same transcript the waiter sees:

```bash
omni pane read <target> --source recent-unwrapped --lines 80
```

That's the matched text. If your needle isn't there, the wait won't fire — adjust the match string or increase `--lines` (if you're hitting the scrollback budget).

## Timeouts and exit codes

- `--timeout MS` — wall-clock milliseconds. The default is generous (600000 = 10 minutes for `agent-status`, 30000 = 30s for `output`); set it explicitly when scripting.
- On timeout, the command exits **1** and prints an error JSON. On match, exits **0**.

Use a long timeout (`600000`+) for `wait agent-status --status done` when the agent might be doing real work. Short timeouts only for known-fast signals like "server ready".

## Already-satisfied edge case

For `wait agent-status`, if the target is **already** at the requested status when the wait starts, the wait fires immediately with an initial event derived from the probe snapshot. No need to set up the wait before triggering the action.

For `wait output`, the initial probe also runs a match against current scrollback. So a recent line that matches will fire the wait immediately — useful for "wait for the last 'ready' to appear" patterns where you can't be sure if you missed it.

## Robust coordination pattern

When orchestrating multiple agents, the typical pattern is:

1. Capture the peer's durable `terminal_id` from `omni pane list` or the spawn response.
2. Dispatch the task with `omni pane run <terminal_id> "..."` or `omni agent send <name> "..."`.
3. Block on `omni wait agent-status <terminal_id> --status done --timeout 600000`.
4. Collect with `omni pane read <terminal_id> --source recent --lines 200`.

Step 1 happens once. Steps 2–4 can repeat for multiple round trips with the same `terminal_id` — it stays valid for the agent's lifetime regardless of layout changes elsewhere.

## When to skip `wait` entirely

If you just want to know "did the agent print X at any point so far?" — that's a `pane read --source recent-unwrapped` question, not a `wait` question. `wait output` is for the **next** occurrence after the wait starts (with the initial probe as a one-shot retroactive check). Use `pane read` for historical inspection.
