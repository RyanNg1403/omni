---
name: omni
description: "Control omni from inside it. Manage workspaces and tabs, split panes, spawn agents, read output, and wait for state changes — all via CLI commands that talk to the running omni instance over a local unix socket. Use when running inside omni (OMNI_ENV=1)."
---

# omni — agent skill

before using this skill, check that `OMNI_ENV=1`. if it is not set to `1`, say you are not running inside an omni-managed pane and stop. do not inspect or control the focused omni pane from outside omni.

you are running inside omni, a terminal-native agent multiplexer. omni gives you workspaces, tabs, and panes — each pane is a real terminal with its own shell, agent, server, or log stream — and you can control all of it from the cli.

the `omni` binary is available in your PATH. its workspace, tab, pane, agent, and wait commands talk to the running omni instance over a local unix socket.

## map of this skill

this file is the hub. read it first. for depth on a specific topic, also read:

- **`references/agents.md`** — when orchestrating named agents, spawning with `--new-tab` / `--split`, or addressing peers by terminal_id / agent name. covers the full `omni agent <subcommand>` family.
- **`references/waits.md`** — when a `wait output` or `wait agent-status` isn't firing, when you need long-running coordination, or when you want to understand how subscriptions survive pane-id compaction.
- **`references/recipes.md`** — multi-step orchestration patterns: round-trip a task, fan-out parallel work, spawn-and-coordinate.

## concepts

**workspaces** are project contexts. each workspace has one or more tabs. unless manually renamed, a workspace's label follows the first tab's root pane — usually the repo name, otherwise the root pane's current folder name.

**tabs** are subcontexts inside a workspace. each tab has one or more panes.

**panes** are terminal splits inside a tab. each pane runs its own process — a shell, an agent, a server, anything.

**agent status** is detected automatically by omni. the api exposes one public field for it:

- `agent_status` — `idle`, `working`, `blocked`, `done`, `unknown`

`done` means the agent finished, but you have not looked at that finished pane yet.

plain shells still exist as panes, but omni's sidebar agent section intentionally focuses on detected agents rather than listing every shell.

**ids** — workspace ids look like `1`, `2`. tab ids look like `1:1`, `1:2`, `2:1`. pane ids look like `1-1`, `1-2`, `2-1`. these are compact public ids for the current live session.

important: ids can compact when tabs, panes, or workspaces are closed. do not treat them as durable ids. re-read ids from `workspace list`, `tab list`, `pane list`, or create/split responses when you need a current id. do not guess that an older `1-3` is still the same pane later.

## terminal ids (durable peer addressing)

every pane is backed by a terminal with a stable `terminal_id` of the form `term_<hex>` (e.g. `term_652d9f4cd425b1`). terminal ids are allocated once per terminal and never reused — they do **not** compact when sibling panes close.

`omni pane list`, `omni pane get`, `omni pane read`, and the `omni agent` command family all expose `terminal_id` alongside `pane_id`. event payloads from `omni wait output` and `omni wait agent-status` also carry it.

every `omni pane` and `omni wait` command that takes a target accepts the `term_<hex>` form interchangeably with the public `w<ws>-N` form:

```bash
omni pane read term_652d9f4cd425b1 --source recent --lines 80
omni pane run  term_652d9f4cd425b1 "echo hello"
omni wait agent-status term_652d9f4cd425b1 --status done --timeout 600000
```

`omni wait` subscriptions resolve the target to a `terminal_id` once at probe time and match incoming events by `terminal_id`, so a wait survives mid-flight compaction.

prefer terminal ids over public pane ids for:

- long-lived references stored in scripts or notes
- targets passed between agents over the socket
- any `omni wait` whose target's sibling panes may close before the wait fires

the public `w<ws>-N` and short `1-2` forms are still fine for quick interactive typing and for one-shot commands you fire immediately after `omni pane list`.

## discover yourself

see what panes exist and which one is focused:

```bash
omni pane list
```

the focused pane is yours. other panes are your neighbors.

list workspaces:

```bash
omni workspace list
```

your own pane id is also available in the `OMNI_PANE_ID` env var (durable `p_<raw>` form).

## tab management

```bash
omni tab list --workspace 1
omni tab create --workspace 1 --label "logs"   # without --label: numbered name
omni tab rename 1:2 "logs"
omni tab focus 1:2
omni tab close 1:2
```

## read another pane

```bash
omni pane read 1-1 --source recent --lines 50
```

- `--source visible` = current viewport
- `--source recent` = recent scrollback as rendered in the pane
- `--source recent-unwrapped` = recent terminal text with soft wraps joined back together

## split a pane and run a command

```bash
omni pane split 1-2 --direction right --no-focus
```

that prints json with the new pane nested at `result.pane.pane_id`. parse that value, then run a command in that pane:

```bash
NEW_PANE=$(omni pane split 1-2 --direction right --no-focus | python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["pane"]["pane_id"])')
omni pane run "$NEW_PANE" "npm run dev"
```

split downward with `--direction down`.

## send text or keys to a pane

```bash
omni pane send-text 1-1 "hello"          # text only, no Enter
omni pane send-keys 1-1 Enter            # press Enter (or other keys)
omni pane run 1-1 "echo hello"           # text + Enter in one request
```

## wait for output

block until specific text appears in a pane. useful for servers, builds, tests.

```bash
omni wait output 1-3 --match "ready on port 3000" --timeout 30000
omni wait output 1-3 --match "server.*ready" --regex --timeout 30000
```

matching uses unwrapped recent terminal text, so pane width and soft wrapping do not break matches. if a wait isn't firing, inspect the same transcript the waiter sees:

```bash
omni pane read 1-3 --source recent-unwrapped --lines 80
```

on timeout, exit code is `1`.

for deeper coordination patterns, robust waits across compaction, and the already-satisfied edge case, read `references/waits.md`.

## wait for an agent status

block until another agent reaches a specific status:

```bash
omni wait agent-status 1-1 --status done --timeout 600000
```

this uses the same `done` / `idle` distinction the sidebar shows. for long-running coordination, prefer the durable `term_<hex>` target so the wait survives sibling pane closures.

## starting agents and addressing them by name

the `omni agent` family is the durable orchestration entry point. brief reference:

```bash
omni agent start reviewer --new-tab --no-focus -- codex   # single-pane tab, named
omni agent send reviewer "review the fix in bug.md"
omni agent wait reviewer --status idle --timeout 600000
omni agent read reviewer --source recent --lines 200
```

for the full `omni agent` reference (target forms, all subcommands, `--new-tab` vs `--split` vs `--tab`, response shapes, when to use `agent` vs `pane`), read `references/agents.md`.

## workspace management

```bash
omni workspace create --cwd /path/to/project --label "api server"   # --label optional
omni workspace create --no-focus
omni workspace focus 2
omni workspace rename 1 "api server"
omni workspace close 2
```

## close a pane

```bash
omni pane close 1-3
```

## notes

- `workspace list`, `workspace create`, `tab list`, `tab create`, `tab get`, `tab focus`, `tab rename`, `tab close`, `pane list`, `pane get`, `pane split`, `wait output`, and `wait agent-status` print json on success.
- `agent list`, `agent get`, `agent start`, `agent rename`, `agent focus` print json on success. `agent send` and `agent attach` print nothing.
- `agent read` prints text, not json (same as `pane read`).
- `pane read` prints text, not json.
- `pane read --format ansi` or `pane read --ansi` returns a rendered ANSI snapshot for TUI feedback loops.
- `pane read --source recent-unwrapped` is useful when you want to inspect the same unwrapped transcript that `wait output --source recent` matches against.
- `pane send-text`, `pane send-keys`, and `pane run` print nothing on success.
- parse ids from `workspace create`, `tab create`, and `pane split` responses when you need new ids. `workspace create` returns `result.workspace`, `result.tab`, and `result.root_pane`. `tab create` returns `result.tab` and `result.root_pane`. for `pane split`, the new pane id is at `result.pane.pane_id`.
- use `pane read` for current output that already exists. use `wait output` for future output you expect next.
- `--no-focus` on split, tab create, and workspace create keeps your current terminal context focused.
- without `--label`, workspace create keeps cwd-based naming and tab create keeps numbered naming.
- `--label` on tab create and workspace create applies the custom name immediately.
- if you are running inside omni, the `OMNI_ENV` environment variable is set to `1`.
