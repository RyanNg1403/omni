---
name: omni
description: "Control omni from inside it. Manage workspaces and tabs, split panes, spawn agents, read output, and wait for state changes — all via CLI commands that talk to the running omni instance over a local unix socket. Use when running inside omni (OMNI_ENV=1)."
---

# omni — agent skill

before using this skill, check that `OMNI_ENV=1`. if it is not set to `1`, say you are not running inside a omni-managed pane and stop. do not inspect or control the focused omni pane from outside omni.

you are running inside omni, a terminal-native agent multiplexer. omni gives you workspaces, tabs, and panes — each pane is a real terminal with its own shell, agent, server, or log stream — and you can control all of it from the cli.

this means you can:

- see what other panes and agents are doing
- create tabs for separate subcontexts inside one workspace
- split panes and run commands in them
- start servers, watch logs, and run tests in sibling panes
- wait for specific output before continuing
- wait for another agent to finish
- spawn more agent instances

the `omni` binary is available in your PATH. its workspace, tab, pane, and wait commands talk to the running omni instance over a local unix socket.

if you need the raw protocol or full api reference, read the [socket api docs](https://github.com/RyanNg1403/omni/blob/main/docs/socket-api.md).

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

## tab management

list tabs in the current workspace:

```bash
omni tab list --workspace 1
```

create a new tab:

```bash
omni tab create --workspace 1
```

without `--label`, the new tab keeps the default numbered tab name.

create and name it in one step:

```bash
omni tab create --workspace 1 --label "logs"
```

rename it:

```bash
omni tab rename 1:2 "logs"
```

focus it:

```bash
omni tab focus 1:2
```

close it:

```bash
omni tab close 1:2
```

## read another pane

see what is on another pane's screen:

```bash
omni pane read 1-1 --source recent --lines 50
```

- `--source visible` = current viewport
- `--source recent` = recent scrollback as rendered in the pane
- `--source recent-unwrapped` = recent terminal text with soft wraps joined back together

## split a pane and run a command

split your pane to the right and keep focus on your current pane:

```bash
omni pane split 1-2 --direction right --no-focus
```

that prints json with the new pane nested at `result.pane.pane_id`. parse that value, then run a command in that pane:

```bash
NEW_PANE=$(omni pane split 1-2 --direction right --no-focus | python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["pane"]["pane_id"])')
omni pane run "$NEW_PANE" "npm run dev"
```

split downward instead:

```bash
omni pane split 1-2 --direction down --no-focus
```

## wait for output

block until specific text appears in a pane. useful for waiting on servers, builds, and tests.

for `--source recent`, matching uses unwrapped recent terminal text, so pane width and soft wrapping do not break matches. `pane read --source recent` still shows the pane as rendered. if you want to inspect the same transcript that the waiter matches, use `pane read --source recent-unwrapped`.

```bash
omni wait output 1-3 --match "ready on port 3000" --timeout 30000
```

with regex:

```bash
omni wait output 1-3 --match "server.*ready" --regex --timeout 30000
```

if it times out, exit code is `1`.

## wait for an agent status

block until another agent reaches a specific status:

```bash
omni wait agent-status 1-1 --status done --timeout 60000
```

use this when you want the same `done` / `idle` distinction the UI shows.

## send text or keys to a pane

send text without pressing Enter:

```bash
omni pane send-text 1-1 "hello from claude"
```

press Enter or other keys:

```bash
omni pane send-keys 1-1 Enter
```

`pane run` sends the text and then a real `Enter` key in one request:

```bash
omni pane run 1-1 "echo hello"
```

## agent commands

the `omni agent` subcommand group targets agents (the running process inside a pane) rather than the pane itself. it accepts a broader set of target forms than `omni pane`:

- the durable `term_<hex>` terminal id (survives public-pane-id compaction)
- a unique agent name set at `agent start`
- a detected agent label (e.g. `codex`, `claude`) when unambiguous
- any pane id the `omni pane` commands accept

spawn a named agent in its own new tab — no shell + split sibling, single-pane tab:

```bash
omni agent start reviewer --new-tab --no-focus -- codex
```

spawn a named agent by splitting the current pane:

```bash
omni agent start worker --split right --no-focus -- claude
```

`--new-tab` is mutually exclusive with `--tab` and `--split`. with `--workspace <id>` the new tab lands in that workspace; without it the active workspace is used.

target an agent by name from anywhere:

```bash
omni agent send reviewer "review the fix in bug-report.md"
omni agent read reviewer --source recent --lines 200
omni agent wait reviewer --status idle --timeout 600000
```

`agent send` writes literal text only — follow up with `omni pane send-keys <target> Enter` if you need the keystroke. for a one-shot "send text + Enter", use `omni pane run` instead.

list all agents, get one, rename, focus, or attach:

```bash
omni agent list
omni agent get reviewer
omni agent rename reviewer code-reviewer
omni agent rename reviewer --clear         # frees the name, keeps the process
omni agent focus reviewer
omni agent attach reviewer
```

when to use `omni agent` vs `omni pane`:

- `omni agent` for orchestration that should survive pane-id shifts: starting named agents, sending tasks, waiting for status, reading output.
- `omni pane` for layout-aware operations: splitting, closing, focusing by position, sending raw keys.

## workspace management

create a new workspace:

```bash
omni workspace create --cwd /path/to/project
```

without `--label`, the new workspace keeps the default cwd-based name.

create and name one in one step:

```bash
omni workspace create --cwd /path/to/project --label "api server"
```

create one without focusing it:

```bash
omni workspace create --no-focus
```

focus a workspace:

```bash
omni workspace focus 2
```

rename:

```bash
omni workspace rename 1 "api server"
```

close:

```bash
omni workspace close 2
```

## close a pane

```bash
omni pane close 1-3
```

## recipes

### run a server and wait until it is ready

```bash
NEW_PANE=$(omni pane split 1-2 --direction right --no-focus | python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["pane"]["pane_id"])')
omni pane run "$NEW_PANE" "npm run dev"
omni wait output "$NEW_PANE" --match "ready" --timeout 30000
omni pane read "$NEW_PANE" --source recent --lines 20
```

### run tests in a separate pane and inspect the result

```bash
omni pane split 1-2 --direction down --no-focus
omni pane run 1-3 "cargo test"
omni wait output 1-3 --match "test result" --timeout 60000
omni pane read 1-3 --source recent --lines 30
```

### check what another agent is working on

```bash
omni pane list
omni pane read 1-1 --source recent --lines 80
```

### watch another pane robustly

use this pattern when you need to coordinate with a sibling pane:

```bash
# inspect what is already there
omni pane read 1-3 --source recent --lines 40

# wait only for the next output you expect
omni wait output 1-3 --match "ready" --timeout 30000

# if you need to inspect the same transcript the waiter matched,
# read the unwrapped recent text directly
omni pane read 1-3 --source recent-unwrapped --lines 40
```

### spawn a new agent and give it a task

```bash
omni pane split 1-2 --direction right --no-focus
omni pane run 1-3 "claude"
omni wait output 1-3 --match ">" --timeout 15000
omni pane run 1-3 "review the test coverage in src/api/"
```

### coordinate with another agent

```bash
omni wait agent-status 1-1 --status done --timeout 120000
omni pane read 1-1 --source recent --lines 100
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
