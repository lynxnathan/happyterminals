# happyterminals-dsl

Declarative builder for [happyterminals](https://github.com/lynxnathan/happyterminals)
scenes: a `react-three-fiber`-shaped tree of typed nodes whose props can be plain
values, `Signal<T>`, or `Memo<T>`. Ships a JSON recipe loader that validates
against a `schemars`-generated schema via `jsonschema`, then produces the
identical `SceneIr` as the Rust builder path.

Dual-licensed under MIT OR Apache-2.0.
