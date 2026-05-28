# skill/recipes.md — multi-step orchestration patterns

This file collects the common multi-step recipes for omni. Each recipe combines primitives covered in the hub `SKILL.md` (and optionally `skill/agents.md`, `skill/waits.md`).

## Run a server and wait until it is ready

```bash
NEW_PANE=$(omni pane split 1-2 --direction right --no-focus | python3 -c 'import sys,json; print(json.load(sys.stdin)["result"]["pane"]["pane_id"])')
omni pane run "$NEW_PANE" "npm run dev"
omni wait output "$NEW_PANE" --match "ready" --timeout 30000
omni pane read "$NEW_PANE" --source recent --lines 20
```

## Run tests in a sibling pane and inspect the result

```bash
omni pane split 1-2 --direction down --no-focus
omni pane run 1-3 "cargo test"
omni wait output 1-3 --match "test result" --timeout 60000
omni pane read 1-3 --source recent --lines 30
```

## Check what another agent is working on

```bash
omni pane list
omni pane read 1-1 --source recent --lines 80
```

## Watch another pane robustly

```bash
# inspect what is already there
omni pane read 1-3 --source recent --lines 40

# wait only for the next output you expect
omni wait output 1-3 --match "ready" --timeout 30000

# if you need to inspect the same transcript the waiter matched,
# read the unwrapped recent text directly
omni pane read 1-3 --source recent-unwrapped --lines 40
```

## Round-trip a task to a named agent

This is the canonical orchestration pattern. The peer is addressed by stable identity (`term_<hex>` or unique agent name) so the round-trip survives layout changes.

```bash
# 1. dispatch
omni pane run <peer_term_id> "review the fix in BUG_REPORT.md and write CODEX_IMPL_REVIEW.md"

# 2. block until the peer reports done
omni wait agent-status <peer_term_id> --status done --timeout 600000

# 3. collect the result
omni pane read <peer_term_id> --source recent --lines 200
```

For long-running reviews use the agent-name form so you can run multiple round trips without re-capturing pane ids:

```bash
omni agent send reviewer "round 2: address findings in CODEX_IMPL_REVIEW.md"
omni agent wait reviewer --status idle --timeout 600000
omni agent read reviewer --source recent --lines 200
```

## Fan out parallel work and gather

Spawn N workers, dispatch in parallel, wait on each, gather output.

```bash
# spawn workers
P1=$(omni pane split "$OMNI_PANE_ID" --direction right --no-focus | jq -r .result.pane.pane_id)
P2=$(omni pane split "$P1" --direction down --no-focus | jq -r .result.pane.pane_id)
P3=$(omni pane split "$P2" --direction down --no-focus | jq -r .result.pane.pane_id)

# capture stable terminal_ids so the references survive any compaction
T1=$(omni pane get "$P1" | jq -r .result.pane.terminal_id)
T2=$(omni pane get "$P2" | jq -r .result.pane.terminal_id)
T3=$(omni pane get "$P3" | jq -r .result.pane.terminal_id)

# dispatch
omni pane run "$T1" "cargo test"
omni pane run "$T2" "cargo clippy"
omni pane run "$T3" "cargo build --release"

# gather
for t in "$T1" "$T2" "$T3"; do
  omni wait agent-status "$t" --status done --timeout 600000
  omni pane read "$t" --source recent --lines 40
done
```

## Spawn a new agent in its own tab and give it a task

The `--new-tab` placement gives the new agent a clean tab of its own (no shell + split sibling).

```bash
omni agent start reviewer --new-tab --no-focus -- codex
# capture its terminal_id for later use
T=$(omni agent get reviewer | jq -r .result.agent.terminal_id)

omni agent wait reviewer --status idle --timeout 30000   # wait for codex to be ready
omni agent send reviewer "review the test coverage in src/api/"
omni pane send-keys "$T" Enter
omni agent wait reviewer --status idle --timeout 600000
omni agent read reviewer --source recent --lines 200
```

## Coordinate with an existing agent (no spawning)

```bash
omni wait agent-status 1-1 --status done --timeout 120000
omni pane read 1-1 --source recent --lines 100
```
