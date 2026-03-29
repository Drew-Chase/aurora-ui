use aurora_ui::aurora_widgets::components::*;
use aurora_ui::prelude::*;

pub fn page_code_block() -> impl Widget {
    col!()
        .spacing(24.0)
        .padding(Edges::new(0.0, 24.0, 0.0, 0.0))
        .child(crate::page_header(
            "Code Block",
            "Renders code with syntax highlighting and line numbers.",
        ))
        .child(crate::example_section("Rust", "A Rust code example."))
        .child(crate::example_card(
            code_block::CodeBlock::new()
                .language("rust")
                .code(
                    r#"use std::collections::HashMap;

/// A simple key-value store.
pub struct Store {
    data: HashMap<String, i64>,
}

impl Store {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn insert(&mut self, key: &str, value: i64) {
        self.data.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&i64> {
        self.data.get(key)
    }
}"#,
                )
                .font_size(13.0),
        ))
        .child(crate::example_section(
            "JavaScript",
            "A JavaScript code example.",
        ))
        .child(crate::example_card(
            code_block::CodeBlock::new()
                .language("javascript")
                .code(
                    r#"import * as React from "react"

export function Counter() {
  const [count, setCount] = React.useState(0)

  return (
    <button onClick={() => setCount(count + 1)}>
      Count: {count}
    </button>
  )
}"#,
                )
                .font_size(13.0),
        ))
        .child(crate::example_section("JSON", "A JSON data example."))
        .child(crate::example_card(
            code_block::CodeBlock::new()
                .language("json")
                .code(
                    r#"{
  "name": "aurora-ui",
  "version": "0.1.0",
  "features": ["syntax", "themes"],
  "dependencies": {
    "regex": "1.10",
    "serde": { "version": "1.0", "features": ["derive"] }
  },
  "count": 42,
  "enabled": true
}"#,
                )
                .font_size(13.0),
        ))
}
