# ls
Structured context:
```json
{"repo":"nanoref (nref) — database-free reference linking, Rust + clap-derive CLI"}
```

Add a `--json` flag to the `nref ls` subcommand.
- Declare it on the `Commands::ls` variant in `src/main.rs` as `#[arg(long)] json: bool`.
- Thread it into `src/ls.rs`'s `run`; when set, print machine-readable JSON instead of the
  human-formatted output. Match the JSON shape to what the command already reports.
- Keep the default (no flag) output byte-identical to today.
Then re-run `cargo build` and `target/debug/nref ls --json` to confirm.
