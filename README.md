<p align="center">
  <img src="assets/gray-logo.svg" alt="gray" width="96">
</p>
<h1 align="center">gray-bookmark</h1>
<p align="center">Named session markers you can `gray resume` around.</p>
<p align="center">
  <a href="https://github.com/vstaln/gray-bookmark/blob/main/LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="gray plugin" src="https://img.shields.io/badge/gray-plugin-7aa2f7.svg">
  <img alt="rust" src="https://img.shields.io/badge/built%20with-rust-orange.svg">
</p>

Named markers you can `gray resume` around — `/bookmark` commands plus a
`bookmark` tool.

## Commands

- `/bookmark <label>` — mark where you are: appends `{label, session, cwd, ts}`
  to `~/.gray/bookmarks.jsonl`
- `/bookmark` — with the `host.ask` capability granted, opens a "Jump
  where?" picker (`<n> <label>` with session id, cwd and timestamp as each
  option's description, plus "cancel"); picking one answers with `Resume
  that session with: `gray resume <sid>` (cwd: <cwd>)`. With no ask
  channel it falls back to the list.
- `/bookmarks` — list all bookmarks (numbered)
- `/bookmark rm <n>` — delete by 1-based index

## Tools

`bookmark` — `{action: "list"|"add"|"remove", label?, index?}`, the same three
verbs for the model. `add` auto-generates `bookmark-<ts>` when `label` is
omitted.

## Honest note

Gray's sidecar wire has no session-entry label or fork/jump API — this
stores named markers
(label + session id + cwd + timestamp) you can `gray resume` around, nothing
more. It does not (cannot) rewind or jump the live session.

## State

`~/.gray/bookmarks.jsonl` (honors `$GRAY_HOME`) — one JSON object per line,
append-only, rewritten on remove.

## Wire methods

`plugin/manifest`, `tool/call`, `command/run`, `plugin/shutdown`, plus
sidecar→host `host/ask` for the bare `/bookmark` picker. Protocol 1.1.
Capability: `host.ask` (the command degrades to the text list without it).

## Install

```sh
gray plugin install bookmark
gray plugin capabilities bookmark --all   # grants host.ask → /bookmark picker
```

## Develop

```sh
cargo test
gray account check      # entry point + manifest handshake
gray account publish    # check → build → release → publish to the gray registry
```

---
Part of the [gray](https://github.com/vstaln/gray) plugin ecosystem —
the open-source AI agent harness. <https://gray.alignment.id>
