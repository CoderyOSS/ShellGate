# OpsxDeliberate Workflow Design

## Architecture

Sequential multi-agent deliberation. Agents review OpenSpec change proposals in
round-robin fashion. Each agent reviews the previous agent's proposals, agrees or
disagrees, and may add new proposals. The cycle continues until all agents agree
or max turns are exceeded.

The primary agent acts as orchestrator — it does NOT analyze artifacts itself.
All analysis and review happens in subagent dispatches.

## Pipeline

```
User: /opsx-deliberate <change-name>
  |
  +-- STATE 0: SETUP
  |     detect change -> verify artifacts -> detect opsx-* agents ->
  |     user selects agents -> set max_turns -> init state file
  |     GATE: artifacts must exist, at least one agent selected
  |
  +-- STATE 1: INITIAL REVIEW
  |     first agent reads all artifacts -> produces proposals ->
  |     write proposals to state file -> turn = 1
  |     GATE: state file written with at least one proposal (or confirmed empty)
  |
  +-- STATE 2: CONSENSUS CYCLE (re-entrant)
  |     next agent reads artifacts + current proposals ->
  |     reviews each: agree or disagree ->
  |     may add new proposals ->
  |     turn++ -> update state file ->
  |     check consensus:
  |       all agents in last full cycle had zero disagreements AND zero new proposals
  |       -> STATE 3 (consensus reached)
  |       turn >= max_turns
  |       -> STATE 3 (forced halt)
  |       else -> handoff to next agent -> STATE 2
  |     GATE: state file updated after each turn
  |
  +-- STATE 3: PRESENT TO USER
  |     formatted output of final proposals + consensus status ->
  |     user: accept / modify / reject
  |     GATE: user has responded
  |
  +-- STATE 4: APPLY (if accepted or modified)
  |     apply approved proposals to artifact files -> commit
  |     GATE: changes applied
  |
  +-- STATE 5: SUMMARY
        final report -> done
```

## Consensus Detection

After each agent's turn in STATE 2:

1. Collect the agent's results: disagreements count + new proposals count
2. Examine the last N turns (N = number of participating agents)
3. If all N turns have zero disagreements AND zero new proposals: consensus reached
4. If fewer than N turns have occurred: cannot reach consensus yet

This means consensus requires one full cycle where every agent reviewed the current
state and had nothing to add or object to.

## State Persistence

State file: `.openspec/tmp/deliberate-state.json`

Contains:
- Change name, max turns, current turn counter
- Agent list and current agent index for rotation
- Complete history of all rounds
- Current proposal pool with statuses and agreement tracking
- Consensus flag

The state file enables crash recovery — if the session is interrupted, the user
can say "continue deliberating" and the agent reads the file to resume from the
current state.

## Agent Rotation

Agents rotate in order: agents[0], agents[1], ..., agents[N-1], agents[0], ...

```
agent_index = (agent_index + 1) % agents.length
```

The first agent (turn 1) does initial review. All subsequent agents do consensus
cycle review. The agent that did the initial review will also participate in the
consensus cycle on its second pass.

## Commit Behavior

Single commit after user accepts. Unlike opsx-refine (which commits per tier),
opsx-deliberate commits once because changes are applied as a batch after user
acceptance.

Commit message: `opsx-deliberate: apply N proposals for <change-name>`

## Turn Counter

Incremented by 1 after each agent dispatch. The counter tracks total dispatches,
not cycles. With 3 agents and 2 full cycles: turn counter = 6 (not 2).

Max turns default: 3 * agents.length

## Dispute Resolution

When an agent disagrees:
- The original proposal is marked "disputed"
- The counter-proposal is added with status "counter"
- Both remain in the proposal pool
- Subsequent agents review both and can agree with either

When the user is presented with disputed proposals:
- Both the original and counter-proposal are shown
- User picks which one to apply (or neither)

## Cost Profile

- **Minimum** (1 agent, zero issues): 1 dispatch
- **Typical** (3 agents, consensus in 1 cycle): 4 dispatches (1 initial + 3 consensus)
- **Maximum** (3 agents, max turns 9): 9 dispatches
- Each dispatch is one subagent call via `task` tool
- Dispatches are sequential (not parallel) — wall-clock time is sum of all dispatches
