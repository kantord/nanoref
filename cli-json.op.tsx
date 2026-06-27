// cli-json.op.tsx — INVARIANT: every nref subcommand exposes a `--json` flag.
import { Context, h, prompt, sh, unit } from 'esto'

sh`cargo build -q`
const BIN = 'target/debug/nref'

const COMMANDS: string[] = JSON.parse(sh`cargo +nightly -Zscript scripts/subcommands.rs`)

const hasJson = (cmd: string): boolean => sh`${BIN} ${cmd} --help`.includes('--json')

const JsonFlag = unit({
  key: (c: { name: string }): string => c.name,
  value: (): string => 'present',
  observe: (): { name: string }[] => COMMANDS.filter(hasJson).map((name) => ({ name })),
  enter: (c: { name: string }) =>
    prompt`Add a \`--json\` flag to the \`nref ${c.name}\` subcommand.
- Declare it on the \`Commands::${c.name}\` variant in \`src/main.rs\` as \`#[arg(long)] json: bool\`.
- Thread it into \`src/${c.name}.rs\`'s \`run\`; when set, print machine-readable JSON instead of the
  human-formatted output. Match the JSON shape to what the command already reports.
- Keep the default (no flag) output byte-identical to today.
Then re-run \`cargo build\` and \`${BIN} ${c.name} --json\` to confirm.`,
})

export default (): unknown => (
  <Context data={{ repo: 'nanoref (nref) — database-free reference linking, Rust + clap-derive CLI' }}>
    {COMMANDS.map((name) => <JsonFlag name={name} />)}
  </Context>
)
