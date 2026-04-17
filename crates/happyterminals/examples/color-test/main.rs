//! Color-test — Phase 2.2 exit artifact.
//!
//! Developer utility — not a demo. Renders a static scene under a chosen or
//! auto-detected color mode so contributors can visually compare truecolor /
//! 256 / 16 / mono output when debugging the color cascade.
//!
//! Usage:
//!
//!     cargo run --example color-test -p happyterminals
//!     cargo run --example color-test -p happyterminals -- --force-color=truecolor
//!     cargo run --example color-test -p happyterminals -- --force-color=256
//!     cargo run --example color-test -p happyterminals -- --force-color=16
//!     cargo run --example color-test -p happyterminals -- --force-color=none
//!
//! Use `NO_COLOR=1 cargo run --example color-test -p happyterminals` to
//! verify the no-color.org env-var pathway.
//!
//! Quits on Ctrl-C (`TerminalGuard` restores the terminal).

use happyterminals::prelude::*;

/// Parse `--force-color=VALUE` out of the raw CLI args. Returns `None`
/// when the flag is absent (auto-detection path). Exits `2` on an
/// unrecognized value — the same convention CLI-arg crates follow,
/// implemented via `std::env::args()` + match to keep deps at zero.
fn parse_force_color() -> Option<ColorMode> {
    for arg in std::env::args().skip(1) {
        if let Some(rest) = arg.strip_prefix("--force-color=") {
            return match rest {
                "truecolor" => Some(ColorMode::TrueColor),
                "256" => Some(ColorMode::Palette256),
                "16" => Some(ColorMode::Ansi16),
                "none" => Some(ColorMode::Mono),
                other => {
                    eprintln!(
                        "color-test: unknown --force-color value '{other}' \
                         (expected: truecolor, 256, 16, none)"
                    );
                    std::process::exit(2);
                }
            };
        }
    }
    None
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let override_mode = parse_force_color();

    // Derive an on-screen provenance label. For `None` we show
    // "auto-detect" without claiming a specific resolved mode — the
    // cascade runs inside `run()` and we'd have to mirror it here to
    // label precisely. Keeping this static avoids the duplication.
    let (header_mode_str, provenance) = match override_mode {
        Some(m) => (format!("{m:?}"), "forced via --force-color".to_string()),
        None => (
            "auto-detect".to_string(),
            "detected at runtime ($NO_COLOR / $COLORTERM / $TERM)".to_string(),
        ),
    };

    run(
        move |grid, _input| {
            // Row 0: mode header.
            let header = format!(
                " color-test - mode: {header_mode_str} ({provenance})  |  Ctrl-C to quit "
            );
            grid.put_str(0, 0, &header, Style::default());

            // Row 2: horizontal RGB gradient — visible across all modes.
            let width = grid.area.width;
            for col in 0..width {
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let tval = f32::from(col) / f32::from(width.max(1));
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let red = (tval * 255.0) as u8;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let green = ((1.0 - tval) * 255.0) as u8;
                let style = Style::default().fg(Color::Rgb(red, green, 128));
                grid.put_str(col, 2, "#", style);
            }

            // Row 4: named 16-color palette — round-trip check for ansi16
            // and palette256 modes (should pass through unchanged).
            let named = [
                (Color::Red, "R"), (Color::Green, "G"), (Color::Blue, "B"),
                (Color::Yellow, "Y"), (Color::Magenta, "M"), (Color::Cyan, "C"),
                (Color::White, "W"),
            ];
            for (idx, (color, label)) in named.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                let col = (idx * 3) as u16;
                grid.put_str(col, 4, label, Style::default().fg(*color).add_modifier(Modifier::BOLD));
            }

            // Row 6: modifier sampler — bold / italic / underline / reversed.
            // Mono strips color but preserves these; quick visual check.
            let mods = [
                (Modifier::BOLD, "BOLD"),
                (Modifier::ITALIC, "ITALIC"),
                (Modifier::UNDERLINED, "UNDER"),
                (Modifier::REVERSED, "REVRS"),
            ];
            let mut cursor: u16 = 0;
            for (modifier, label) in mods {
                let style = Style::default().fg(Color::Rgb(200, 200, 255)).add_modifier(modifier);
                grid.put_str(cursor, 6, label, style);
                #[allow(clippy::cast_possible_truncation)]
                let span = (label.len() + 1) as u16;
                cursor = cursor.saturating_add(span);
            }
        },
        FrameSpec {
            title: Some("happyterminals - color-test".into()),
            color_mode: override_mode,
            ..FrameSpec::default()
        },
    )
    .await
}
