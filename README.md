# gray-bookmark

Named markers you can `gray resume` around — `/bookmark` commands plus a
`bookmark` tool. Port of pi's `bookmark` extension.

## Commands

- `/bookmark <label>` — mark where you are: appends `{label, session, cwd, ts}`
  to `~/.gray/bookmarks.jsonl`
- `/bookmark` or `/bookmarks` — list all bookmarks (numbered)
- `/bookmark rm <n>` — delete by 1-based index

## Tools

`bookmark` — `{action: "list"|"add"|"remove", label?, index?}`, the same three
verbs for the model. `add` auto-generates `bookmark-<ts>` when `label` is
omitted.

## Honest note

Pi's version labeled session entries for `/tree` navigation. Gray's sidecar
wire has no session-entry label or fork/jump API — this stores named markers
(label + session id + cwd + timestamp) you can `gray resume` around, nothing
more. It does not (cannot) rewind or jump the live session.

## State

`~/.gray/bookmarks.jsonl` (honors `$GRAY_HOME`) — one JSON object per line,
append-only, rewritten on remove.

## Wire methods

`plugin/manifest`, `tool/call`, `command/run`, `plugin/shutdown`. Protocol 1.1.
No capabilities, no host→sidecar requests.

## Install

```sh
gray plugin install bookmark
```

## Develop

```sh
cargo test
gray account check      # entry point + manifest handshake
gray account publish    # check → build → release → publish to the gray registry
```
