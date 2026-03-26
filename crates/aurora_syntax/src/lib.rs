//! Regex-based syntax highlighting with JSON language definitions.
//!
//! Built-in support for Rust, JavaScript, TypeScript, Python, C, C++,
//! JSON, HTML, XML, CSS, Markdown, Go, Java, TOML, YAML, and Bash.
//!
//! # Example
//!
//! ```
//! use aurora_syntax::{SyntaxSet, TokenType};
//!
//! let syntax = SyntaxSet::default();
//! let tokens = syntax.highlight("let x = 42;", "rust");
//! for (range, token_type) in &tokens {
//!     println!("{:?}: {:?}", token_type, &"let x = 42;"[range.clone()]);
//! }
//! ```

use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::OnceLock;

/// Token types produced by the highlighter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Keyword,
    String,
    Comment,
    Number,
    Function,
    Type,
    Operator,
    Punctuation,
    Attribute,
    Tag,
    Constant,
    Plain,
}

/// A compiled language definition with ordered regex patterns.
pub struct Language {
    /// Patterns in priority order (comments/strings first).
    rules: Vec<(TokenType, Regex)>,
}

/// Collection of compiled languages.
pub struct SyntaxSet {
    languages: HashMap<String, Language>,
}

#[derive(Deserialize)]
struct LanguageDef(HashMap<String, String>);

/// Priority order for pattern matching. Earlier entries take precedence
/// so comments and strings mask everything inside them.
const TOKEN_PRIORITY: &[(&str, TokenType)] = &[
    ("comment", TokenType::Comment),
    ("string", TokenType::String),
    ("attribute", TokenType::Attribute),
    ("keyword", TokenType::Keyword),
    ("constant", TokenType::Constant),
    ("type", TokenType::Type),
    ("function", TokenType::Function),
    ("number", TokenType::Number),
    ("tag", TokenType::Tag),
    ("operator", TokenType::Operator),
    ("punctuation", TokenType::Punctuation),
];

impl Language {
    fn from_def(def: &LanguageDef) -> Self {
        let mut rules = Vec::new();
        for &(name, token_type) in TOKEN_PRIORITY {
            if let Some(pattern) = def.0.get(name) {
                // Compile with multiline mode so ^ and $ match line boundaries
                if let Ok(re) = Regex::new(&format!("(?m){pattern}")) {
                    rules.push((token_type, re));
                }
            }
        }
        Self { rules }
    }
}

impl SyntaxSet {
    /// Loads languages from a JSON string.
    pub fn from_json(json: &str) -> Self {
        let defs: HashMap<String, LanguageDef> =
            serde_json::from_str(json).unwrap_or_default();
        let mut languages = HashMap::new();
        for (name, def) in &defs {
            languages.insert(name.clone(), Language::from_def(def));
        }
        Self { languages }
    }

    /// Returns the built-in language set (loaded once, cached).
    pub fn builtin() -> &'static Self {
        static INSTANCE: OnceLock<SyntaxSet> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            Self::from_json(include_str!("languages.json"))
        })
    }

    /// Highlights code, returning non-overlapping byte ranges with token types.
    ///
    /// Patterns are applied in priority order (comments/strings first).
    /// Overlapping matches are skipped to prevent highlighting inside strings
    /// or comments.
    pub fn highlight(&self, code: &str, language: &str) -> Vec<(Range<usize>, TokenType)> {
        let Some(lang) = self.languages.get(language) else {
            return Vec::new();
        };

        let mut tokens: Vec<(Range<usize>, TokenType)> = Vec::new();
        // Track occupied byte positions to prevent overlapping matches
        let mut occupied = vec![false; code.len()];

        for (token_type, regex) in &lang.rules {
            for mat in regex.find_iter(code) {
                let range = mat.start()..mat.end();
                // Skip if any byte in this range is already claimed
                if range.clone().any(|i| occupied[i]) {
                    continue;
                }
                // For function patterns with capture groups, use group 1 if available
                if *token_type == TokenType::Function
                    && let Some(caps) = regex.captures(&code[mat.start()..mat.end()])
                    && let Some(m) = caps.get(1)
                {
                    let abs_start = mat.start() + m.start();
                    let abs_end = mat.start() + m.end();
                    for byte in &mut occupied[abs_start..abs_end] {
                        *byte = true;
                    }
                    tokens.push((abs_start..abs_end, *token_type));
                    continue;
                }
                for byte in &mut occupied[range.clone()] {
                    *byte = true;
                }
                tokens.push((range, *token_type));
            }
        }

        // Sort by start position for efficient lookup
        tokens.sort_by_key(|(r, _)| r.start);
        tokens
    }

    /// Returns the list of available language names.
    pub fn languages(&self) -> Vec<&str> {
        self.languages.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for SyntaxSet {
    fn default() -> Self {
        Self::builtin().clone_languages()
    }
}

impl SyntaxSet {
    fn clone_languages(&self) -> Self {
        // Re-parse from JSON since Language contains Regex which isn't Clone
        Self::from_json(include_str!("languages.json"))
    }
}
