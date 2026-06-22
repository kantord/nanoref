# nanoref

**nanoref** is a database-free system for information linking using global references — you
embed the same opaque, grep-able marker in two or more places to connect them, and listing
every occurrence of a marker with ripgrep IS the link graph.

## How it works

You embed a marker as plain text anywhere — a code comment, a Markdown comment, an HTML/XML
attribute, a log message, anywhere a short alphanumeric token can live.  Placing the **same
marker** in multiple files (or multiple places in the same file) means those places are
linked.  Listing all occurrences of that marker shows everything connected to it.

There is **no registry, no database, and no index** — the whole system is strings in files
plus a text search tool.  Markers are globally-unique opaque random IDs, so anyone can mint
them independently with negligible collision risk, and separate projects or repositories can
be merged without coordination.

## Marker format

```
nref-<check><body>
```

| Part | Length | Description |
|------|--------|-------------|
| Prefix | 5 | The literal `nref-` |
| `<check1><check2>` | 2 | Version-tagged checksum characters (see below) |
| `<body>` | 21 | Random payload (nanoid) |

**Alphabet** — nanoid's default URL-safe set: `A-Za-z0-9_-` (64 symbols).

The body contains 21 random characters (~126 bits of entropy), giving negligible collision
probability for uncoordinated global minting.

**Full marker length:** `nref-` (5) + checks (2) + body (21) = **28 characters total**.

Example:

```
nref-OWaEnfYlRhkxxYNoSKAsfjy
```

### Canonical regex

```
\bnref-[A-Za-z0-9_-]{23}\b
```

### Check characters

`<check1>` and `<check2>` are `SHA-256(salt + body)[0] % 64` and `SHA-256(salt + body)[1] % 64`
respectively, each mapped to the alphabet above.  The salt for v1 markers is the ASCII
string `nref-version-1`.

Two characters (12 bits) give a 1/4096 ≈ 0.024% false-negative rate for both corruption
detection and version discrimination.

This serves two purposes:
1. **Integrity** — a single-character corruption anywhere in the body produces a different
   check with overwhelming probability.
2. **Version tagging** — future nref versions use a different salt, so `validate` can
   identify which version minted a given marker without storing version metadata separately.

SHA-256 is available as a standard library in every major language (`hashlib` in Python,
`node:crypto` in Node.js, `crypto/sha256` in Go, `java.security.MessageDigest` in Java)
with no third-party dependencies and no patent encumbrances.

## Usage

### `nanoref ls [PATH]`

Recursively search PATH (default: current directory) for all nanoref markers and print the
link graph.

```
nanoref ls [PATH]
```

For each distinct marker found, `ls` prints the marker, its total occurrence count, and the
`path:line` of every occurrence — sorted deterministically.  Markers appearing in more than
one place reveal cross-file links at a glance.

**Example output:**

```
nref-OWaEnfYlRhkxxYNoSKAsfjy (2)
  docs/design.md:14
  src/main.rs:3
nref--spd9runA1h_5k6PnhjcA4r (1)
  README.md:22
```

The walk respects `.gitignore` and skips hidden files/directories by default (same behaviour
as ripgrep).

## Installation

```sh
cargo install --path .
```

## Design notes

- **No central registry** — any contributor mints markers locally; the only coordination is
  "don't reuse someone else's marker", which the 126-bit body makes statistically safe.
- **Plain text** — markers survive copy-paste, reformatting, and most diffs intact.
- **Composable** — `nanoref ls` output is plain text; pipe it, grep it, script it.
- **Merge-safe** — merging two repos that each minted their own markers introduces no
  conflicts; just run `nanoref ls` in the merged tree.
