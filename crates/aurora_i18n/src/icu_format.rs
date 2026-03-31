use icu_decimal::FixedDecimalFormatter;
use icu_decimal::options::FixedDecimalFormatterOptions;
use icu_locale::Locale;

/// Locale-aware number formatter backed by ICU4X.
///
/// Provides comprehensive locale coverage via the Unicode CLDR data set.
/// For a lightweight alternative that covers common locales without the
/// binary size cost, see [`crate::format::format_number`].
///
/// # Example
///
/// ```ignore
/// use aurora_i18n::icu_format::IcuNumberFormatter;
///
/// let fmt = IcuNumberFormatter::new("de-DE");
/// assert_eq!(fmt.format_i64(1234567), "1.234.567");
/// ```
pub struct IcuNumberFormatter {
    inner: FixedDecimalFormatter,
}

impl IcuNumberFormatter {
    /// Creates a number formatter for the given BCP 47 locale tag.
    ///
    /// Falls back to `"en-US"` if the locale cannot be parsed.
    pub fn new(locale_tag: &str) -> Self {
        let locale: Locale = locale_tag
            .parse()
            .unwrap_or_else(|_| "en-US".parse().unwrap());
        let inner =
            FixedDecimalFormatter::try_new(locale.into(), FixedDecimalFormatterOptions::default())
                .expect("ICU data available for locale");
        Self { inner }
    }

    /// Formats an integer with locale-appropriate grouping separators.
    pub fn format_i64(&self, value: i64) -> String {
        let fixed = icu_decimal::FixedDecimal::from(value);
        self.inner.format_to_string(&fixed)
    }

    /// Formats a float with locale-appropriate decimal and grouping separators.
    ///
    /// `max_fraction_digits` controls how many decimal places are shown.
    pub fn format_f64(&self, value: f64, max_fraction_digits: i16) -> String {
        let mut fixed =
            icu_decimal::FixedDecimal::try_from_f64(value, icu_decimal::FloatPrecision::Floating)
                .unwrap_or_default();
        fixed.trunc(max_fraction_digits);
        self.inner.format_to_string(&fixed)
    }
}
