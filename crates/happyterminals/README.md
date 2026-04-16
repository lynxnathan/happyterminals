# happyterminals

Meta crate for the [happyterminals](https://github.com/lynxnathan/happyterminals)
scene manager. Re-exports the curated public surface under `happyterminals::prelude::*`.

Most users should depend on this crate rather than the individual sub-crates.

## Terminal Color Support

happyterminals renders in truecolor (24-bit RGB) by default. For terminals
that don't support truecolor, it gracefully degrades to xterm's 256-color
palette, then to the 16 ANSI system colors, and finally to monochrome with
text modifiers (bold, italic, underline, reversed) preserved.

### Detection cascade

At `run()` entry, the active color mode is resolved once via this
priority order:

1. **`NO_COLOR` environment variable** — if present and **non-empty**
   (any value), color is disabled and the output is monochrome.
   Empty string (`NO_COLOR=""`) is treated as unset per the
   [no-color.org](https://no-color.org/) spec.
2. **`FrameSpec.color_mode`** — programmatic override; wins over
   `$COLORTERM` / `$TERM` but not over `NO_COLOR`.
3. **`$COLORTERM`** — if it equals `truecolor` or `24bit`
   (case-insensitive), truecolor is used. Other values (including the
   legacy `COLORTERM=1`) fall through.
4. **`$TERM` fallback**:
   - `dumb` → monochrome
   - any value containing `256color` → 256-color palette
   - `xterm-kitty` or `alacritty` → truecolor
   - anything else → 16-color ANSI
   - unset → monochrome (piped / non-TTY output)

Detection happens exactly once at startup. There are no per-frame env
reads and no silent overrides — if you force a mode, happyterminals
respects it literally.

### Forcing a color mode

Set `FrameSpec.color_mode` when building your app:

```rust,ignore
use happyterminals::prelude::*;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    run(
        |grid, _input| { /* ... */ },
        FrameSpec {
            color_mode: Some(ColorMode::Ansi16),
            ..FrameSpec::default()
        },
    )
    .await
}
```

The four `ColorMode` variants are `TrueColor`, `Palette256`, `Ansi16`,
and `Mono`. `FrameSpec::default().color_mode` is `None`, meaning
runtime auto-detection.

### Verifying color output

The `color-test` example lets you eyeball each mode:

```sh
cargo run --example color-test -p happyterminals
cargo run --example color-test -p happyterminals -- --force-color=truecolor
cargo run --example color-test -p happyterminals -- --force-color=256
cargo run --example color-test -p happyterminals -- --force-color=16
cargo run --example color-test -p happyterminals -- --force-color=none
```

Source: [`examples/color-test/main.rs`](examples/color-test/main.rs).

### tmux: pass truecolor through

tmux multiplexes many panes over one underlying terminal, which means
it needs an explicit opt-in to forward 24-bit color escapes instead of
quantizing them down to the 256-color palette. If you run
happyterminals inside tmux and see muted / banded colors even though
your underlying terminal supports truecolor, add the following to your
`~/.tmux.conf`:

```conf
# Default to a 256-color-capable TERM inside tmux.
set -g default-terminal "tmux-256color"

# Pass truecolor (24-bit RGB) escapes through to the underlying terminal.
set -ga terminal-overrides ",*256col*:Tc"
```

Reload tmux (`tmux source-file ~/.tmux.conf` or restart) and
happyterminals inside tmux will see `$COLORTERM=truecolor` and emit
24-bit SGR escapes. Without `Tc`, tmux silently strips those escapes
to 256-color approximations — the common source of "colors look wrong
inside tmux" reports.

happyterminals does not auto-probe tmux (`$TMUX` etc.). If you override
to `ColorMode::TrueColor` inside a tmux session that lacks `Tc`, you'll
get the strip-down; the fix is in your tmux config, not in our library.

Further reading:

- [no-color.org](https://no-color.org/) — canonical `NO_COLOR` spec.
- [termstandard/colors](https://github.com/termstandard/colors) —
  `$COLORTERM=truecolor|24bit` de facto standard.
- [Sunaku's tmux 24-bit color guide](https://sunaku.github.io/tmux-24bit-color.html) —
  deeper dive on the `Tc` flag.

## License

Dual-licensed under MIT OR Apache-2.0.
