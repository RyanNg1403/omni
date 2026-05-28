# omni

<p align="center">
  <img src="assets/logo.png" alt="omni" width="100" />
</p>

<p align="center">
  <a href="#install">install</a> · <a href="#quick-start">quick start</a> · <a href="#supported-agents">supported agents</a>
</p>

> **omni is a fork of [herdr](https://github.com/ogulcancelik/herdr) by Ogulcan Celik**, released under the same AGPL-3.0-or-later license. omni follows its own design philosophy and is maintained independently. See [NOTICE](./NOTICE) for full attribution and the `omni-fork-point` tag for the upstream divergence point.

---

https://github.com/user-attachments/assets/043ec09f-4bdd-41d5-aee0-8fda6b83e267

**agent multiplexer that lives in your terminal.**

workspaces, tabs, panes. mouse-native: click, drag, split. every agent at a glance: blocked, working, done. detach and reattach, agents keep running. no gui app, no electron, no mac-only native wrapper. you see the agent's own terminal, not someone's interpretation of it.

---

## install

```bash
curl -fsSL https://github.com/RyanNg1403/omni | sh
```

or install with homebrew:

```bash
brew install omni
```

or download the binary from [releases](https://github.com/RyanNg1403/omni/releases). requires linux or macos.

### update

omni notifies you when a new version is available. run manually to update:

```bash
omni update
```

By default, updating installs the new binary and leaves compatible running sessions alone, or asks before stopping sessions that must restart. To opt into live server handoff for supported running sessions, run `omni update --handoff`.

`omni update` is disabled for Homebrew and Nix installs. Update those through `brew upgrade omni` or your Nix workflow; live handoff does not apply to package-manager updates.

## quick start

```bash
omni
```

by default omni launches or attaches to one background session server. `ctrl+b q` detaches the client. agents keep running. use `omni server stop` to stop the default server. use `--no-session` for the old single-process mode.

named sessions are runtime/socket namespaces for separate persistent omni servers. they do not replace workspaces; each named session has its own panes, tabs, workspaces, sockets, and session state while sharing the same global config file.

```bash
omni session list
omni session attach work
omni session attach side-project
omni session stop work
omni session delete side-project
```

1. press `ctrl+b`, then `shift+n` to create a workspace
2. run an agent in the root pane
3. press `ctrl+b`, then `w` to open workspace navigation
4. use `ctrl+b`, then `v` or `minus` to split panes, or `ctrl+b`, then `c` to create a new tab
5. watch the sidebar for blocked, working, and done states

on first run omni opens a short onboarding flow. after that, restored sessions land in terminal mode; fresh sessions start in **navigate mode**.

## how it compares

|                          | tmux | gui managers | omni |
|--------------------------|------|--------------|-------|
| persistent sessions       | ✓    | —            | ✓     |
| detach / reattach        | ✓    | —            | ✓     |
| panes, tabs, workspaces  | ✓    | ✓            | ✓     |
| agent awareness          | —    | ✓            | ✓     |
| lives in your terminal   | ✓    | —            | ✓     |
| real terminal views      | ✓    | —            | ✓     |
| mouse-native            | —    | ✓            | ✓     |
| lightweight binary       | ✓    | —            | ✓     |
| agents can orchestrate   | ?    | ?            | ✓     |

tmux gives you persistence and panes, but it was built before agents existed. gui managers show agent state, but they make you leave your terminal and use their wrapped view. omni is persistence and awareness in one tool that stays out of your way.

## persistence

start omni where the work lives. locally, run `omni`. it starts or attaches to the background session automatically, with no socket setup. run your agents, split panes, do your work. press `ctrl+b q` to detach. close your terminal, close your laptop; your agents keep running. open a new terminal, run `omni`, you're back. same session, same panes, same agents.

if you stop the server and later start omni again, omni restores the saved session shape. pane screen history, native agent session restore, and live handoff cover different restart/update cases; see [session state and restore](https://github.com/RyanNg1403/omni).

### from anywhere

need to check on your agents from your phone? just ssh in and run omni. your shell is remote, omni runs there, and the panes keep running there after detach. any ssh client works. no app to download, no account to create.

```
ssh you@yourserver
omni
```

or attach from your local terminal through ssh without opening a shell first. your local omni acts as a thin client, connects over ssh, starts or attaches to the remote omni server, and streams the ui back to your terminal. remote attach uses your local keybindings by default; pass `--remote-keybindings server` to use the remote server config instead. pass `--handoff` to opt into live handoff if remote attach needs to replace a supported running remote server. Homebrew and Nix clients bootstrap remote hosts from the matching release asset instead of copying the package-manager-managed local binary.

```bash
omni --remote workbox
omni --remote ssh://you@yourserver:2222
```

for repeat targets, use your ssh config:

```sshconfig
Host workbox
  HostName yourserver
  User you
  Port 2222
```

same session, same agents, same state.

### direct agent attach

`omni` and `omni --remote` attach to the full omni session ui. `omni agent attach <target>` attaches your current terminal directly to one server-owned terminal, like a single-pane terminal attach. `omni terminal attach <terminal_id>` does the same by terminal id.

direct attach streams the current rendered terminal state first, then live ansi frames. your input goes straight to that terminal. scroll with the mouse wheel or plain page up/page down; normal input jumps back to the bottom. detach with `ctrl+b q`; send a literal `ctrl+b` with `ctrl+b ctrl+b`. one writable client owns input and resize for a terminal. a second attach fails unless you pass `--takeover`.

## agent awareness

the sidebar shows which agents are blocked, working, or done. workspaces roll up to their most urgent state so you can scan the full list at a glance.

states:

- 🔴 **blocked** — agent needs input or approval
- 🟡 **working** — agent is actively running
- 🔵 **done** — work finished, you have not looked at it yet
- 🟢 **idle** — done and seen

detection works by reading foreground process and terminal output. zero config, no hooks required. for agents that expose hooks, the socket api integration gives more robust state reporting.

## lives in your terminal

not a gui window, not a web dashboard, not electron. omni runs inside whatever terminal you already use. single rust binary, no dependencies. works inside tmux.

## what you get

- **workspaces** — organized around git repos or folder names, each with its own tabs and panes
- **tabs** — first-class in the socket api and cli
- **mouse-native** — click panes/tabs/workspaces/agents, drag borders, drag-select text to copy, double-click tokens to copy, right-click menus; not keyboard-only
- **notifications** — sounds and toasts for background events; tab-aware suppression
- **18 built-in themes** — catppuccin, terminal, tokyo night, gruvbox, one, solarized, kanagawa, rosé pine, vesper, and light variants for the main palettes
- **session persistence** — pane processes survive client detach; sessions restore panes after full restart, with opt-in recent screen history

## agents can use omni too

the local unix socket lets agents create workspaces, split panes, spawn helpers, read output, and wait for state changes.

```bash
# create a workspace and tab
omni workspace create --cwd ~/project --label "api"
omni tab create --label "logs"

# split a pane and run
omni pane split 1-1 --direction right
omni pane run 1-2 "npm test"

# wait for a pane-level ui attention state
omni wait agent-status 1-1 --status done

# read output
omni pane read 1-2 --source recent --lines 50

# read a rendered ansi snapshot for tui feedback loops
omni pane read 1-2 --source visible --ansi
```

full reference: [socket api](https://github.com/RyanNg1403/omni) and [`SKILL.md`](./SKILL.md).

## supported agents

automatic detection works out of the box. process name matching plus terminal output heuristics.

| agent | idle / done | working | blocked |
|-------|-------------|---------|---------|
| [pi](https://pi.dev) | ✓ | ✓ | partial |
| [claude code](https://docs.anthropic.com/en/docs/claude-code) | ✓ | ✓ | ✓ |
| [codex](https://github.com/openai/codex) | ✓ | ✓ | ✓ |
| [droid](https://factory.ai) | ✓ | ✓ | ✓ |
| [amp](https://ampcode.com) | ✓ | ✓ | ✓ |
| [opencode](https://github.com/anomalyco/opencode) | ✓ | ✓ | ✓ |
| [grok cli](https://x.ai/grok) | ✓ | ✓ | ✓ |
| [hermes agent](https://github.com/NousResearch/hermes-agent) | ✓ | ✓ | ✓ |
| cursor agent | ✓ | ✓ | ✓ |
| antigravity cli | ✓ | ✓ | ✓ |
| kimi code cli | ✓ | ✓ | ✓ |
| [github copilot cli](https://github.com/features/copilot) | ✓ | ✓ | ✓ |
| qoder cli | ✓ | ✓ | ✓ |
| [kiro cli](https://kiro.dev/docs/cli/) | ✓ | ✓ | — |

detected but not fully tested: gemini cli, cline.

for agents outside the built-in list, omni still works as a terminal multiplexer with workspaces, panes, and tiling. custom integrations can report agent labels over the socket api. see the [socket api docs](https://github.com/RyanNg1403/omni).

### direct integrations

the built-in pi, omp, claude code, codex, opencode, hermes, and qoder cli integrations forward semantic state to omni over the socket api. install with:

```bash
omni integration install pi
omni integration install omp
omni integration install claude
omni integration install codex
omni integration install opencode
omni integration install hermes
omni integration install qodercli
```

see the [integrations docs](https://github.com/RyanNg1403/omni) for setup details.

## keybindings

press `ctrl+b` to enter prefix mode. default actions are prefix-first and tmux-like:

| key | action |
|-----|--------|
| `prefix+c` | new tab |
| `prefix+n` / `prefix+p` | next / previous tab |
| `prefix+1..9` | switch tab |
| `prefix+w` | workspace navigation |
| `prefix+g` | session navigator |
| `prefix+shift+n` | new workspace |
| `prefix+shift+g` | new worktree |
| `prefix+shift+w` | rename workspace |
| `prefix+shift+d` | close workspace |
| `prefix+h/j/k/l` | focus pane |
| `prefix+v` / `prefix+minus` | split pane |
| `prefix+x` | close pane |
| `prefix+b` | toggle sidebar |
| `prefix+z` | zoom pane |
| `prefix+r` | resize mode |
| `prefix+q` | detach |

resize mode: `h`/`l` resize width, `j`/`k` resize height, `esc` exit.

session navigator opens a searchable workspace, tab, and pane tree. use `/` for text search, `b`/`w`/`i`/`d` for blocked, working, idle, and done filters, `a` or backspace to clear a state filter, and enter to switch to the highlighted row.

last-pane is available but unset by default. bind `last_pane` in `[keys]` if you want tmux-style back-and-forth navigation to the last focused pane across workspaces and tabs; for example, `last_pane = "prefix+tab"`.

custom command keybindings can launch detached shell helpers or temporary panes:

```toml
[[keys.command]]
key = "prefix+alt+g"
type = "pane" # "shell" or "pane"
command = "lazygit"
```

if you have old custom keybindings and want the new defaults, run `omni config reset-keys`. omni backs up `config.toml`, removes only keybinding config, and uses built-in v2 defaults after restart or config reload.

mouse is supported throughout. full reference: [configuration docs](https://github.com/RyanNg1403/omni).

## configuration

config file: `~/.config/omni/config.toml`

```bash
omni --default-config   # print full default config
```

in-app settings screen for theme, sound, and toast preferences. full reference: [configuration docs](https://github.com/RyanNg1403/omni).

## logs

omni writes logs under `~/.config/omni/`.

common files:

```text
~/.config/omni/omni.log
~/.config/omni/omni-client.log
~/.config/omni/omni-server.log
```

in persistent session mode, the client and server logs are usually the useful ones. logs rotate automatically and keep a few older files like `.1` and `.2`.

for issue reports, include the relevant current log plus rotated siblings if they exist. default logs are metadata-focused and avoid pane contents by default.

use a higher log level only when needed:

```bash
OMNI_LOG=omni=debug omni
```

full logging and environment variable details: [configuration docs](https://github.com/RyanNg1403/omni).

## docs

- [configuration](https://github.com/RyanNg1403/omni) — keybindings, themes, notifications, environment variables
- [integrations](https://github.com/RyanNg1403/omni) — pi, omp, claude code, codex, opencode, hermes, qoder cli integrations
- [`SKILL.md`](./SKILL.md) — reusable agent skill
- [socket api](https://github.com/RyanNg1403/omni) — socket protocol and cli reference

## agent instructions

if you are an ai agent helping with this repository, read [`AGENTS.md`](./AGENTS.md) before making changes and read [`CONTRIBUTING.md`](./CONTRIBUTING.md) before opening issues or PRs.

## building from source

```bash
git clone https://github.com/RyanNg1403/omni
cd omni
cargo build --release
./target/release/omni
```

## nix

omni provides optional nix flake outputs for users who already use nix. the flake builds omni from source.

```bash
nix run github:RyanNg1403/omni/v0.x.y
nix build github:RyanNg1403/omni/v0.x.y
nix develop github:RyanNg1403/omni/v0.x.y
```

replace `v0.x.y` with the latest release tag. you can omit the tag to track `master`, but release tags are recommended for normal installs.

the flake exposes `packages.<system>.default`, `apps.<system>.default`, `devShells.<system>.default`, and `overlays.default`.

update through the same nix workflow you used to install omni. for profile installs, run `nix profile list` and then `nix profile upgrade <index-or-name>`. for flake inputs, run `nix flake update omni` in your own flake and rebuild.

## testing

```bash
just test        # unit tests
just check       # formatting, tests, and maintenance checks
```

## license

Omni is dual-licensed:

1. Open source: GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later).
2. Commercial: commercial licenses are available for organizations that cannot comply with AGPL.

Contact: hey@omni.todo

## pi, ghostty, and shift+enter

omni does not require or install terminal keybinds for pi.

ghostty does not ship a default `shift+enter=text:\n` or `shift+enter=text:\x1b\r` keybind. if those lines exist in your ghostty config, they were added by user config or another tool, commonly claude code. they collapse shift+enter into legacy bytes, so downstream programs cannot reliably distinguish shift+enter from ctrl+j or alt+enter.

if shift+enter behaves differently in pi inside omni, first remove those custom terminal keybinds and retest. do not file this as a omni keyboard encoding bug unless it reproduces with a clean terminal config.

related context: #78, #81, #106, and earendil-works/pi#1872.

## mandatory star history

<a href="https://www.star-history.com/?repos=ogulcancelik%2Fomni&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=RyanNg1403/omni&type=date&theme=dark&legend=top-left&v=2026-05-19" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=RyanNg1403/omni&type=date&legend=top-left&v=2026-05-19" />
   <img alt="star history chart" src="https://api.star-history.com/chart?repos=RyanNg1403/omni&type=date&legend=top-left&v=2026-05-19" />
 </picture>
</a>
