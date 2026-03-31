use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;

/// A collection of localization message bundles keyed by locale tag.
///
/// Wraps [fluent-rs](https://github.com/projectfluent/fluent-rs) to provide
/// message formatting with argument substitution and locale fallback.
///
/// # Example
///
/// ```
/// use aurora_i18n::messages::MessageBundles;
/// use fluent_bundle::FluentValue;
///
/// let mut bundles = MessageBundles::new("en-US");
/// bundles
///     .add_from_str(
///         "en-US",
///         "hello = Hello, { $name }!\ncount = You have { $n } items.",
///     )
///     .unwrap();
///
/// let msg = bundles.format("hello", &[("name", FluentValue::from("World"))]);
/// assert_eq!(msg, "Hello, \u{2068}World\u{2069}!");
/// ```
pub struct MessageBundles {
    default_locale: String,
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl MessageBundles {
    /// Creates a new message bundle collection with a default locale.
    pub fn new(default_locale: impl Into<String>) -> Self {
        Self {
            default_locale: default_locale.into(),
            bundles: HashMap::new(),
        }
    }

    /// Parses a Fluent source string and adds it to the bundle for `locale`.
    ///
    /// Returns an error if the Fluent source fails to parse.
    pub fn add_from_str(
        &mut self,
        locale: &str,
        source: &str,
    ) -> Result<(), Vec<fluent_syntax::parser::ParserError>> {
        let resource = FluentResource::try_new(source.to_string()).map_err(|(_, errors)| errors)?;
        let lang_id: unic_langid::LanguageIdentifier =
            locale.parse().expect("valid BCP 47 locale tag");
        let bundle = self
            .bundles
            .entry(locale.to_string())
            .or_insert_with(|| FluentBundle::new(vec![lang_id]));
        bundle
            .add_resource(resource)
            .map_err(|_| Vec::<fluent_syntax::parser::ParserError>::new())
            .ok();
        Ok(())
    }

    /// Formats a message by ID with the given arguments using the default locale.
    ///
    /// Falls back to the message ID wrapped in braces if no translation is found.
    pub fn format(&self, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        self.format_for_locale(&self.default_locale, id, args)
    }

    /// Formats a message for a specific locale with fallback to the default locale.
    pub fn format_for_locale(
        &self,
        locale: &str,
        id: &str,
        args: &[(&str, FluentValue<'_>)],
    ) -> String {
        let try_format = |bundle: &FluentBundle<FluentResource>| -> Option<String> {
            let msg = bundle.get_message(id)?;
            let pattern = msg.value()?;
            let mut fluent_args = FluentArgs::new();
            for (key, val) in args {
                fluent_args.set(*key, val.clone());
            }
            let mut errors = Vec::new();
            let result = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
            Some(result.into_owned())
        };

        // Try requested locale first
        if let Some(bundle) = self.bundles.get(locale)
            && let Some(result) = try_format(bundle)
        {
            return result;
        }
        // Fallback to default locale
        if locale != self.default_locale
            && let Some(bundle) = self.bundles.get(&self.default_locale)
            && let Some(result) = try_format(bundle)
        {
            return result;
        }
        // Last resort: return the message ID
        format!("{{{id}}}")
    }

    /// Sets the default locale tag.
    pub fn set_default_locale(&mut self, locale: impl Into<String>) {
        self.default_locale = locale.into();
    }

    /// Returns the current default locale tag.
    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_formatting() {
        let mut bundles = MessageBundles::new("en-US");
        bundles.add_from_str("en-US", "greeting = Hello!").unwrap();
        assert_eq!(bundles.format("greeting", &[]), "Hello!");
    }

    #[test]
    fn formatting_with_args() {
        let mut bundles = MessageBundles::new("en-US");
        bundles
            .add_from_str("en-US", "hello = Hello, { $name }!")
            .unwrap();
        let msg = bundles.format("hello", &[("name", FluentValue::from("Aurora"))]);
        assert!(msg.contains("Aurora"));
    }

    #[test]
    fn missing_message_returns_id() {
        let bundles = MessageBundles::new("en-US");
        assert_eq!(bundles.format("missing", &[]), "{missing}");
    }

    #[test]
    fn locale_fallback() {
        let mut bundles = MessageBundles::new("en-US");
        bundles.add_from_str("en-US", "greeting = Hello!").unwrap();
        // Request French, but it doesn't exist — should fall back to en-US
        let msg = bundles.format_for_locale("fr-FR", "greeting", &[]);
        assert_eq!(msg, "Hello!");
    }

    #[test]
    fn multi_locale() {
        let mut bundles = MessageBundles::new("en-US");
        bundles.add_from_str("en-US", "greeting = Hello!").unwrap();
        bundles
            .add_from_str("es-ES", "greeting = \u{00A1}Hola!")
            .unwrap();
        assert_eq!(
            bundles.format_for_locale("es-ES", "greeting", &[]),
            "\u{00A1}Hola!"
        );
        assert_eq!(
            bundles.format_for_locale("en-US", "greeting", &[]),
            "Hello!"
        );
    }
}
