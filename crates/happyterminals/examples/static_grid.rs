//! Static grid demo: renders grapheme-correct text (ASCII + emoji + CJK + ZWJ),
//! exits on Ctrl-C with clean terminal.
//!
//! Run: `cargo run -p happyterminals --example static_grid`

use happyterminals::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        |grid, input_signals| {
            let bold = Style::default().add_modifier(Modifier::BOLD);
            let cyan = Style::default().fg(Color::Cyan);
            let yellow = Style::default().fg(Color::Yellow);
            let green = Style::default().fg(Color::Green);

            grid.put_str(2, 1, "happyterminals Phase 1.1 - Static Grid", bold);
            grid.put_str(2, 3, "ASCII:   Hello, World!", Style::default());
            grid.put_str(2, 4, "Emoji:   \u{1f3a8} \u{1f680} \u{1f3ad} \u{1f308}", cyan);
            grid.put_str(2, 5, "CJK:     \u{4f60}\u{597d}\u{4e16}\u{754c}", yellow);
            grid.put_str(
                2,
                6,
                "ZWJ:     \u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}\u{200d}\u{1f466} family",
                green,
            );

            // Demo: show terminal size from input signals (BACK-03 proof)
            let (w, h) = input_signals.terminal_size.untracked();
            let size_str = format!("Terminal: {w}x{h}");
            grid.put_str(2, 8, &size_str, Style::default());

            grid.put_str(2, 10, "Press Ctrl-C to exit", Style::default());
        },
        FrameSpec::default(),
    )
    .await
}
