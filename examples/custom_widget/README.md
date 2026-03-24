# Custom Widget Example

This example implements the **same toggle switch widget** two different ways, so you can directly compare the approaches.

## Composite Widget (`composite_widget.rs`)

Builds the toggle by **composing existing widgets** — `TouchArea`, `BoxWidget`, `Positioned` — using the `#[composite_widget]` macro for zero-boilerplate builder pattern and `Widget` impl.

You define a config struct, implement `Default` and `CompositeBuilder::build()`, and you're done. The macro generates `new()`, builder setters for all `pub` fields, and the full `Widget` implementation.

```rust
#[composite_widget]
pub struct ToggleSwitch {
    pub on_color: Color,
    pub off_color: Color,
    // ...
}

impl CompositeBuilder for ToggleSwitch {
    fn build(&self) -> Box<dyn Widget> {
        // compose existing widgets here
    }
}

// Usage:
ToggleSwitch::new().on_color(hex!(0x2196F3))
```

This is similar to React components: assemble existing primitives, wire up callbacks to update state, and let the framework re-render.

**Use when:**
- Your widget is a combination of existing widgets with specific styling
- You need state management (hover, toggle, selection, etc.)
- You don't need custom drawing — the built-in primitives cover your visuals
- You want less code and faster iteration

## Full Widget (`full_widget.rs`)

Builds the toggle by **implementing the `Widget` trait directly** — manually handling `layout()`, `paint()`, and `event()`.

This gives you full control: you decide exactly what pixels are drawn, how hit-testing works, and how the widget sizes itself.

**Use when:**
- You need custom drawing that can't be achieved by composing existing widgets
- You want precise control over paint order, clipping, or canvas operations
- You're building a primitive that other widgets will compose (like `BoxWidget` itself)
- Performance matters and you want to avoid the overhead of a widget subtree

## Running

```bash
cargo run -p custom_widget
```
