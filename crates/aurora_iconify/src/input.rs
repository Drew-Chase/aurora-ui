//! Parses the arguments of `icon_sets!`.
//!
//! ```text
//! input := entry ("," entry)* [","]
//! entry := STRING [":" ("all" | "[" [STRING ("," STRING)* [","]] "]")]
//! ```
//!
//! The grammar is small enough to read from the macro input's text form,
//! which keeps this crate free of `syn` and the parser unit-testable.

use crate::names::is_valid_name;

#[derive(Debug, PartialEq)]
pub enum Mode {
    /// Embed the icons the crate refers to, plus these.
    Referenced { always: Vec<String> },
    /// Embed every icon in the set.
    All,
}

#[derive(Debug, PartialEq)]
pub struct SetSpec {
    pub prefix: String,
    pub mode: Mode,
}

const USAGE: &str =
    r#"expected icon set names, e.g. icon_sets!("mage", "lucide": ["arrow-left"], "tabler": all)"#;

pub fn parse(input: &str) -> Result<Vec<SetSpec>, String> {
    let tokens = lex(input)?;
    let mut tokens = tokens.iter().peekable();
    let mut specs: Vec<SetSpec> = Vec::new();

    while let Some(token) = tokens.next() {
        let Token::Str(prefix) = token else {
            return Err(USAGE.into());
        };
        let prefix = name(prefix, "icon set")?;
        if specs.iter().any(|spec| spec.prefix == prefix) {
            return Err(format!("icon set \"{prefix}\" is listed more than once"));
        }

        let mut mode = Mode::Referenced { always: Vec::new() };
        if tokens.next_if_eq(&&Token::Punct(':')).is_some() {
            mode = match tokens.next() {
                Some(Token::Ident(word)) if word == "all" => Mode::All,
                Some(Token::Punct('[')) => {
                    let mut always = Vec::new();
                    loop {
                        match tokens.next() {
                            Some(Token::Punct(']')) => break,
                            Some(Token::Str(icon)) => always.push(name(icon, "icon")?),
                            _ => return Err(USAGE.into()),
                        }
                        match tokens.next() {
                            Some(Token::Punct(']')) => break,
                            Some(Token::Punct(',')) => {}
                            _ => return Err(USAGE.into()),
                        }
                    }
                    Mode::Referenced { always }
                }
                _ => {
                    return Err(format!(
                        "expected `all` or a list of icon names after \"{prefix}\":"
                    ));
                }
            };
        }
        specs.push(SetSpec { prefix, mode });

        match tokens.next() {
            None => break,
            Some(Token::Punct(',')) => {}
            Some(_) => return Err(USAGE.into()),
        }
    }

    if specs.is_empty() {
        return Err("icon_sets! requires at least one icon set name".into());
    }
    Ok(specs)
}

fn name(value: &str, what: &str) -> Result<String, String> {
    if is_valid_name(value) {
        Ok(value.to_string())
    } else {
        Err(format!(
            "invalid {what} name \"{value}\": expected lowercase letters, digits and dashes"
        ))
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Ident(String),
    Punct(char),
}

fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((start, c)) = chars.next() {
        match c {
            c if c.is_whitespace() => {}
            ',' | ':' | '[' | ']' => tokens.push(Token::Punct(c)),
            '"' => {
                let end = loop {
                    match chars.next() {
                        Some((end, '"')) => break end,
                        // Valid names contain nothing that needs escaping.
                        Some((_, '\\')) | None => return Err(USAGE.into()),
                        Some(_) => {}
                    }
                };
                tokens.push(Token::Str(input[start + 1..end].to_string()));
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut end = start + c.len_utf8();
                while let Some((i, c)) = chars.next_if(|(_, c)| c.is_alphanumeric() || *c == '_') {
                    end = i + c.len_utf8();
                }
                tokens.push(Token::Ident(input[start..end].to_string()));
            }
            _ => return Err(USAGE.into()),
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lazy(prefix: &str, always: &[&str]) -> SetSpec {
        SetSpec {
            prefix: prefix.into(),
            mode: Mode::Referenced {
                always: always.iter().map(|s| s.to_string()).collect(),
            },
        }
    }

    #[test]
    fn plain_list_is_still_accepted() {
        assert_eq!(
            parse(r#""mage", "lucide", "material-symbols""#).unwrap(),
            [
                lazy("mage", &[]),
                lazy("lucide", &[]),
                lazy("material-symbols", &[])
            ]
        );
        assert_eq!(parse(r#""mage","#).unwrap(), [lazy("mage", &[])]);
    }

    #[test]
    fn per_set_options() {
        // Spacing as produced by `TokenStream::to_string`.
        let specs =
            parse(r#""mage" , "lucide" : ["arrow-left" , "x" ,] , "tabler" : all"#).unwrap();
        assert_eq!(specs[0], lazy("mage", &[]));
        assert_eq!(specs[1], lazy("lucide", &["arrow-left", "x"]));
        assert_eq!(
            specs[2],
            SetSpec {
                prefix: "tabler".into(),
                mode: Mode::All
            }
        );
        assert_eq!(parse(r#""mage": []"#).unwrap(), [lazy("mage", &[])]);
    }

    #[test]
    fn rejects_bad_input() {
        for input in [
            "",
            "mage",
            r#""mage" "lucide""#,
            r#""mage": everything"#,
            r#""mage": ["a" "b"]"#,
            r#""mage": ["a""#,
            r#""mage", "mage""#,
            r#""../secrets""#,
            r#""Mage""#,
            r#""mage": ["a/b"]"#,
            r#""ma\"ge""#,
            r#"r"mage""#,
            r#""mage"; "lucide""#,
        ] {
            assert!(parse(input).is_err(), "should reject: {input}");
        }
    }
}
