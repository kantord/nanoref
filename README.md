# nanoref

**nanoref** (`nref`) is a database-free system for information linking using global references.
Embed the same opaque, grep-able marker in two or more places to connect them — listing every
occurrence of a marker is the link graph.

There is **no registry, no database, no index**.  Markers are globally-unique random IDs, so
anyone can mint them independently with negligible collision risk.

## Installation

```sh
cargo install nanoref
```

## Commands

### `nref mint` — generate a new marker

```
$ nref mint
nref-vvhnOkm3fFQ75BVIZgbJSeE
```

Paste the output wherever you want to establish a link.  Putting the same marker in two files
connects them.

### `nref ls [PATH]` — show the link graph

Recursively searches PATH (default: `.`) for all markers, groups occurrences by marker, and
prints context lines around each match.  Respects `.gitignore`.

```
nref-dzWVZ98hJTPoQxPgtBsKo8V, 3 location(s)
  src/api.py
  9-# Rate-limit state lives in-process; not suitable for multi-replica deploy.
  10-# See architecture note before scaling horizontally.
  ↪ # nref-dzWVZ98hJTPoQxPgtBsKo8V
  12-_rate_counters: dict[str, tuple[int, float]] = {}
  --
  docs/adr-003.md
  33-## Rate limiting note
  34-
  ↪ In-process rate limiting (nref-dzWVZ98hJTPoQxPgtBsKo8V) shares the same
  36-single-process assumption as this pooling approach.
  --
  docs/known-issues.md
  10-Proper fix: replace the in-process dict with a Redis-backed counter, which
  ↪ also unblocks horizontal scaling (see nref-dzWVZ98hJTPoQxPgtBsKo8V).
```

### `nref check [PATH]` — validate checksums

Recursively searches PATH (default: `.`) for anything that looks like a nanoref marker and
reports corrupted or malformed tokens.  Exits 0 when everything is clean.

```
$ nref check .
src/api.py
↪ nref-XAs1KY-2RTRGIuyKLlhH9b5
error: bad checksum: nref-XAs1KY-2RTRGIuyKLlhH9b5
```

Run this in CI to catch accidental edits to marker text.

## Marker format

```
nref-<check1><check2><body>
```

| Part | Length | Description |
|------|--------|-------------|
| `nref-` | 5 | Fixed prefix |
| `<check1><check2>` | 2 | Version-tagged checksum |
| `<body>` | 21 | Random payload (nanoid) |

**Total: 28 characters.**  Alphabet: nanoid's default URL-safe set `A-Za-z0-9_-` (64 symbols).

The 21-character body gives ~126 bits of entropy — negligible collision probability for
uncoordinated global minting across independent teams and repositories.

### Canonical regex

```
\bnref-[A-Za-z0-9_-]{23}
```

Note: no trailing `\b` — the word-boundary assertion misfires when a marker is immediately
followed by `_` or another word character (both are word chars in most regex flavours).

### Check characters

`check1` and `check2` are `SHA-256(salt + body)[0] % 64` and `SHA-256(salt + body)[1] % 64`
mapped to the alphabet above.  The salt for v1 markers is the ASCII string `nref-version-1`.

Two characters (12 bits) give a 1/4096 ≈ 0.024 % false-negative rate for corruption detection.
Future versions use a different salt, so `nref check` can identify which version minted a
marker without storing extra metadata.

SHA-256 is available in the standard library of every major language — no third-party
dependencies, no patent encumbrances.

## Design notes

- **No central registry** — mint markers locally; 126-bit bodies make conflicts statistically
  impossible across independent contributors.
- **Plain text** — markers survive copy-paste, reformatting, and most diffs intact.
- **Merge-safe** — merging two repos that independently minted markers introduces no conflicts.
- **Composable** — `nref ls` output is plain text; pipe it, grep it, script it.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
