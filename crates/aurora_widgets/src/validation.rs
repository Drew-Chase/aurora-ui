//! Form validation system.
//!
//! Provides a [`Validator`] trait, built-in validators, composable
//! [`ValidatorChain`], and a [`FormValidator`] for multi-field validation.
//!
//! # Example
//!
//! ```
//! use aurora_widgets::validation::*;
//!
//! let chain = ValidatorChain::new()
//!     .with(Required::new())
//!     .with(Email::new());
//!
//! assert!(chain.validate("user@example.com").is_ok());
//! assert!(chain.validate("").is_err());
//! ```

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ValidationResult
// ---------------------------------------------------------------------------

/// The result of a single validation check.
///
/// `Ok(())` means the value is valid. `Err(messages)` contains one or more
/// human-readable error descriptions.
pub type ValidationResult = Result<(), Vec<String>>;

// ---------------------------------------------------------------------------
// Validator trait
// ---------------------------------------------------------------------------

/// A composable field validator.
///
/// Implement this trait for custom validation logic, or use one of the
/// built-in validators ([`Required`], [`MinLength`], [`MaxLength`],
/// [`Email`], [`NumericRange`], [`Pattern`], [`Custom`]).
pub trait Validator: Send + Sync {
    /// Validate the given string value.
    fn validate(&self, value: &str) -> ValidationResult;
}

/// Blanket impl so closures can be used as validators.
impl<F> Validator for F
where
    F: Fn(&str) -> ValidationResult + Send + Sync,
{
    fn validate(&self, value: &str) -> ValidationResult {
        (self)(value)
    }
}

// ---------------------------------------------------------------------------
// Built-in validators
// ---------------------------------------------------------------------------

/// Rejects empty or whitespace-only values.
pub struct Required {
    message: String,
}

impl Required {
    pub fn new() -> Self {
        Self {
            message: "This field is required".to_string(),
        }
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }
}

impl Default for Required {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for Required {
    fn validate(&self, value: &str) -> ValidationResult {
        if value.trim().is_empty() {
            Err(vec![self.message.clone()])
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------

/// Rejects values shorter than `min` characters.
pub struct MinLength {
    min: usize,
    message: String,
}

impl MinLength {
    pub fn new(min: usize) -> Self {
        Self {
            min,
            message: format!("Must be at least {min} characters"),
        }
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }
}

impl Validator for MinLength {
    fn validate(&self, value: &str) -> ValidationResult {
        if value.chars().count() < self.min {
            Err(vec![self.message.clone()])
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------

/// Rejects values longer than `max` characters.
pub struct MaxLength {
    max: usize,
    message: String,
}

impl MaxLength {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            message: format!("Must be at most {max} characters"),
        }
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }
}

impl Validator for MaxLength {
    fn validate(&self, value: &str) -> ValidationResult {
        if value.chars().count() > self.max {
            Err(vec![self.message.clone()])
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------

/// Validates that the value looks like an email address.
///
/// This performs a simple structural check (contains `@` with text on both
/// sides, and a `.` after the `@`). It does not perform DNS or RFC 5322
/// validation.
pub struct Email {
    message: String,
}

impl Email {
    pub fn new() -> Self {
        Self {
            message: "Invalid email address".to_string(),
        }
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }
}

impl Default for Email {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for Email {
    fn validate(&self, value: &str) -> ValidationResult {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            // Empty values should be caught by Required, not Email.
            return Ok(());
        }
        let valid = trimmed
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
        if valid { Ok(()) } else { Err(vec![self.message.clone()]) }
    }
}

// ---------------------------------------------------------------------------

/// Validates that the value parses as a number within `[min, max]`.
pub struct NumericRange {
    min: f64,
    max: f64,
    message: String,
}

impl NumericRange {
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            message: format!("Must be between {min} and {max}"),
        }
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }
}

impl Validator for NumericRange {
    fn validate(&self, value: &str) -> ValidationResult {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        match trimmed.parse::<f64>() {
            Ok(n) if (self.min..=self.max).contains(&n) => Ok(()),
            Ok(_) => Err(vec![self.message.clone()]),
            Err(_) => Err(vec!["Must be a valid number".to_string()]),
        }
    }
}

// ---------------------------------------------------------------------------

/// Validates the value against a regular expression.
///
/// Requires the `regex` feature.
#[cfg(feature = "regex")]
pub struct Pattern {
    re: regex::Regex,
    message: String,
}

#[cfg(feature = "regex")]
impl Pattern {
    /// Creates a new pattern validator.
    ///
    /// # Panics
    ///
    /// Panics if `pattern` is not a valid regular expression.
    pub fn new(pattern: &str, message: impl Into<String>) -> Self {
        Self {
            re: regex::Regex::new(pattern).expect("invalid regex pattern"),
            message: message.into(),
        }
    }
}

#[cfg(feature = "regex")]
impl Validator for Pattern {
    fn validate(&self, value: &str) -> ValidationResult {
        if value.is_empty() {
            return Ok(());
        }
        if self.re.is_match(value) {
            Ok(())
        } else {
            Err(vec![self.message.clone()])
        }
    }
}

// ---------------------------------------------------------------------------

/// Wraps a closure as a named validator.
pub struct Custom<F>
where
    F: Fn(&str) -> ValidationResult + Send + Sync,
{
    f: F,
}

impl<F> Custom<F>
where
    F: Fn(&str) -> ValidationResult + Send + Sync,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> Validator for Custom<F>
where
    F: Fn(&str) -> ValidationResult + Send + Sync,
{
    fn validate(&self, value: &str) -> ValidationResult {
        (self.f)(value)
    }
}

// ---------------------------------------------------------------------------
// ValidatorChain
// ---------------------------------------------------------------------------

/// Chains multiple validators together. All validators run and all errors
/// are collected (not short-circuiting).
pub struct ValidatorChain {
    validators: Vec<Box<dyn Validator>>,
}

impl ValidatorChain {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Appends a validator to the chain.
    pub fn with(mut self, v: impl Validator + 'static) -> Self {
        self.validators.push(Box::new(v));
        self
    }

    /// Runs all validators and collects all errors.
    pub fn validate(&self, value: &str) -> ValidationResult {
        let mut errors = Vec::new();
        for v in &self.validators {
            if let Err(mut errs) = v.validate(value) {
                errors.append(&mut errs);
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl Default for ValidatorChain {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FormValidator
// ---------------------------------------------------------------------------

/// Validates multiple named fields at once.
///
/// # Example
///
/// ```
/// use aurora_widgets::validation::*;
///
/// let form = FormValidator::new()
///     .field("email", ValidatorChain::new().with(Required::new()).with(Email::new()))
///     .field("password", ValidatorChain::new().with(Required::new()).with(MinLength::new(8)));
///
/// let errors = form.validate(&[("email", "bad"), ("password", "short")]);
/// assert!(errors.contains_key("email"));
/// assert!(errors.contains_key("password"));
/// ```
pub struct FormValidator {
    fields: Vec<(String, ValidatorChain)>,
}

impl FormValidator {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
        }
    }

    /// Registers a named field with its validator chain.
    pub fn field(mut self, name: impl Into<String>, chain: ValidatorChain) -> Self {
        self.fields.push((name.into(), chain));
        self
    }

    /// Validates all registered fields against the provided values.
    ///
    /// `values` is a slice of `(field_name, field_value)` pairs. Fields not
    /// present in `values` are validated with an empty string.
    ///
    /// Returns a map of field names to their error messages. Fields that pass
    /// validation are not included in the map.
    pub fn validate(&self, values: &[(&str, &str)]) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        for (name, chain) in &self.fields {
            let value = values
                .iter()
                .find(|(k, _)| *k == name.as_str())
                .map(|(_, v)| *v)
                .unwrap_or("");
            if let Err(errors) = chain.validate(value) {
                result.insert(name.clone(), errors);
            }
        }
        result
    }

    /// Returns `true` if all fields pass validation.
    pub fn is_valid(&self, values: &[(&str, &str)]) -> bool {
        self.validate(values).is_empty()
    }
}

impl Default for FormValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_rejects_empty() {
        assert!(Required::new().validate("").is_err());
        assert!(Required::new().validate("   ").is_err());
    }

    #[test]
    fn required_accepts_nonempty() {
        assert!(Required::new().validate("hello").is_ok());
        assert!(Required::new().validate(" a ").is_ok());
    }

    #[test]
    fn min_length_validates() {
        assert!(MinLength::new(3).validate("ab").is_err());
        assert!(MinLength::new(3).validate("abc").is_ok());
        assert!(MinLength::new(3).validate("abcd").is_ok());
    }

    #[test]
    fn max_length_validates() {
        assert!(MaxLength::new(3).validate("abcd").is_err());
        assert!(MaxLength::new(3).validate("abc").is_ok());
        assert!(MaxLength::new(3).validate("ab").is_ok());
    }

    #[test]
    fn email_validates() {
        assert!(Email::new().validate("user@example.com").is_ok());
        assert!(Email::new().validate("a@b.c").is_ok());
        assert!(Email::new().validate("bad").is_err());
        assert!(Email::new().validate("@example.com").is_err());
        assert!(Email::new().validate("user@").is_err());
        // Empty is OK (use Required to reject empty).
        assert!(Email::new().validate("").is_ok());
    }

    #[test]
    fn numeric_range_validates() {
        let v = NumericRange::new(1.0, 100.0);
        assert!(v.validate("50").is_ok());
        assert!(v.validate("1").is_ok());
        assert!(v.validate("100").is_ok());
        assert!(v.validate("0").is_err());
        assert!(v.validate("101").is_err());
        assert!(v.validate("abc").is_err());
        // Empty is OK.
        assert!(v.validate("").is_ok());
    }

    #[test]
    fn validator_chain_collects_all_errors() {
        let chain = ValidatorChain::new()
            .with(Required::new())
            .with(MinLength::new(5));

        // Empty triggers both Required and MinLength.
        let result = chain.validate("");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn form_validator_maps_errors_by_field() {
        let form = FormValidator::new()
            .field(
                "email",
                ValidatorChain::new()
                    .with(Required::new())
                    .with(Email::new()),
            )
            .field(
                "name",
                ValidatorChain::new().with(Required::new()),
            );

        let errors = form.validate(&[("email", "bad"), ("name", "")]);
        assert!(errors.contains_key("email")); // bad email
        assert!(errors.contains_key("name")); // empty name
        assert_eq!(errors.len(), 2);

        // Valid input.
        let errors = form.validate(&[("email", "a@b.c"), ("name", "Alice")]);
        assert!(errors.is_empty());
    }

    #[test]
    fn custom_validator_works() {
        let v = Custom::new(|value: &str| {
            if value == "secret" {
                Ok(())
            } else {
                Err(vec!["Must be 'secret'".to_string()])
            }
        });
        assert!(v.validate("secret").is_ok());
        assert!(v.validate("other").is_err());
    }

    #[test]
    fn closure_as_validator() {
        let chain = ValidatorChain::new().with(|value: &str| -> ValidationResult {
            if value.contains('!') {
                Ok(())
            } else {
                Err(vec!["Must contain '!'".to_string()])
            }
        });
        assert!(chain.validate("hello!").is_ok());
        assert!(chain.validate("hello").is_err());
    }

    #[test]
    fn form_validator_missing_field_uses_empty() {
        let form = FormValidator::new().field(
            "required_field",
            ValidatorChain::new().with(Required::new()),
        );
        // Field not provided in values — should be validated as empty.
        let errors = form.validate(&[]);
        assert!(errors.contains_key("required_field"));
    }

    #[test]
    fn form_validator_is_valid() {
        let form = FormValidator::new().field(
            "name",
            ValidatorChain::new().with(Required::new()),
        );
        assert!(!form.is_valid(&[("name", "")]));
        assert!(form.is_valid(&[("name", "Alice")]));
    }
}
