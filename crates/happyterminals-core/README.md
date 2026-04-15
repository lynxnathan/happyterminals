# happyterminals-core

Foundational types for [happyterminals](https://github.com/lynxnathan/happyterminals):
reactive primitives (`Signal`, `Memo`, `Effect`, `Owner`) wrapping `reactive_graph`,
and the `Grid` buffer (newtype over `ratatui::Buffer`, grapheme-cluster aware).

Depends on `ratatui-core` only, never the full `ratatui` facade or a backend.
`pyo3` is NOT a dependency of this crate — Python bindings live in
`happyterminals-py`.

Dual-licensed under MIT OR Apache-2.0.
