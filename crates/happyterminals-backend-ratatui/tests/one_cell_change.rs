use ratatui::Terminal;
use ratatui::style::{Color, Style};
use ratatui_crossterm::CrosstermBackend;
use happyterminals_core::Grid;
use std::cell::RefCell;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;

/// A `Write` wrapper around `Rc<RefCell<Vec<u8>>>` so we can share the byte
/// sink between the `CrosstermBackend` and our test assertions.
#[derive(Clone)]
struct SharedBuf(Rc<RefCell<Vec<u8>>>);

impl SharedBuf {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }
    fn len(&self) -> usize {
        self.0.borrow().len()
    }
    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

impl Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn one_cell_change_minimal_bytes() {
    let buf = SharedBuf::new();
    let backend = CrosstermBackend::new(buf.clone());
    let mut terminal = Terminal::new(backend).unwrap();

    // Initial full render
    terminal
        .draw(|frame| {
            let mut grid = Grid::new(frame.area());
            grid.put_str(0, 0, "Hello, World!", Style::default());
            *frame.buffer_mut() = grid.deref().clone();
        })
        .unwrap();

    // Clear byte counter after initial render
    buf.clear();

    // Second render: change exactly one cell
    terminal
        .draw(|frame| {
            let mut grid = Grid::new(frame.area());
            grid.put_str(0, 0, "Hello, World!", Style::default());
            grid.put_str(0, 0, "X", Style::default().fg(Color::Red));
            *frame.buffer_mut() = grid.deref().clone();
        })
        .unwrap();

    let delta_bytes = buf.len();
    // Cursor-move (~6 bytes) + SGR color (~15 bytes) + char (1 byte) + reset (~15 bytes)
    // Empirically measured at ~44 bytes. Threshold: <= 50 bytes.
    // A full-buffer repaint of 80x24 would be thousands of bytes, so this proves
    // ratatui's diff engine works with our Grid (no full repaints).
    assert!(
        delta_bytes <= 50,
        "Expected <= 50 bytes for 1-cell change, got {delta_bytes}"
    );
    // Also verify it's not zero (something was actually written)
    assert!(
        delta_bytes > 0,
        "Expected some bytes for 1-cell change, got 0"
    );
}
