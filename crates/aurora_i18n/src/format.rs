use crate::locale::Locale;

/// Locale-specific formatting patterns.
struct LocalePattern {
    /// Decimal separator (e.g. `.` or `,`)
    decimal: char,
    /// Thousands grouping separator (e.g. `,` or `.` or ` `)
    grouping: char,
    /// Date order: 0 = MDY, 1 = DMY, 2 = YMD
    date_order: u8,
    /// Date separator (e.g. `/` or `.` or `-`)
    date_sep: char,
}

/// Returns the formatting pattern for the given locale tag.
/// Falls back to US English for unknown locales.
fn pattern_for(tag: &str) -> LocalePattern {
    let primary = tag.split(['-', '_']).next().unwrap_or("en");
    let region = tag.split(['-', '_']).nth(1).unwrap_or("");

    match (primary, region) {
        // US English: 1,234.56 / MM/DD/YYYY
        ("en", "US" | "") => LocalePattern {
            decimal: '.',
            grouping: ',',
            date_order: 0,
            date_sep: '/',
        },
        // UK English: 1,234.56 / DD/MM/YYYY
        ("en", _) => LocalePattern {
            decimal: '.',
            grouping: ',',
            date_order: 1,
            date_sep: '/',
        },
        // German, French, Spanish, Portuguese, Italian, Arabic, Turkish, etc.
        ("de", _) => LocalePattern {
            decimal: ',',
            grouping: '.',
            date_order: 1,
            date_sep: '.',
        },
        ("fr", _) | ("ru", _) => LocalePattern {
            decimal: ',',
            grouping: ' ',
            date_order: 1,
            date_sep: '/',
        },
        ("es", _) | ("pt", _) | ("it", _) | ("tr", _) | ("nl", _) => LocalePattern {
            decimal: ',',
            grouping: '.',
            date_order: 1,
            date_sep: '/',
        },
        ("ar", _) | ("fa", _) | ("ur", _) | ("he", _) => LocalePattern {
            decimal: '.',
            grouping: ',',
            date_order: 1,
            date_sep: '/',
        },
        // CJK: 1,234.56 / YYYY-MM-DD
        ("ja", _) | ("zh", _) | ("ko", _) => LocalePattern {
            decimal: '.',
            grouping: ',',
            date_order: 2,
            date_sep: '/',
        },
        // Nordic
        ("sv", _) | ("nb", _) | ("da", _) | ("fi", _) => LocalePattern {
            decimal: ',',
            grouping: ' ',
            date_order: 2,
            date_sep: '-',
        },
        // Polish, Czech, Hungarian
        ("pl", _) | ("cs", _) | ("hu", _) => LocalePattern {
            decimal: ',',
            grouping: ' ',
            date_order: 1,
            date_sep: '.',
        },
        // Hindi
        ("hi", _) => LocalePattern {
            decimal: '.',
            grouping: ',',
            date_order: 1,
            date_sep: '/',
        },
        // Default: US English
        _ => LocalePattern {
            decimal: '.',
            grouping: ',',
            date_order: 0,
            date_sep: '/',
        },
    }
}

/// Formats a float with locale-appropriate decimal and grouping separators.
///
/// # Example
/// ```
/// use aurora_i18n::locale::Locale;
/// use aurora_i18n::format::format_number;
///
/// let en = Locale::new("en-US");
/// assert_eq!(format_number(1234567.89, 2, &en), "1,234,567.89");
///
/// let de = Locale::new("de-DE");
/// assert_eq!(format_number(1234567.89, 2, &de), "1.234.567,89");
/// ```
pub fn format_number(value: f64, decimal_places: u32, locale: &Locale) -> String {
    let pat = pattern_for(locale.tag());
    let abs = value.abs();
    let integer_part = abs.trunc() as u64;
    let frac_part = ((abs - abs.trunc()) * 10f64.powi(decimal_places as i32)).round() as u64;

    // Format integer with grouping
    let int_str = integer_part.to_string();
    let grouped = add_grouping(&int_str, pat.grouping);

    let sign = if value < 0.0 { "-" } else { "" };

    if decimal_places == 0 {
        format!("{sign}{grouped}")
    } else {
        let frac_str = format!("{:0>width$}", frac_part, width = decimal_places as usize);
        format!("{sign}{grouped}{}{frac_str}", pat.decimal)
    }
}

/// Formats an integer with locale-appropriate grouping separators.
///
/// # Example
/// ```
/// use aurora_i18n::locale::Locale;
/// use aurora_i18n::format::format_integer;
///
/// let en = Locale::new("en-US");
/// assert_eq!(format_integer(1234567, &en), "1,234,567");
/// ```
pub fn format_integer(value: i64, locale: &Locale) -> String {
    let pat = pattern_for(locale.tag());
    let abs = value.unsigned_abs();
    let int_str = abs.to_string();
    let grouped = add_grouping(&int_str, pat.grouping);
    let sign = if value < 0 { "-" } else { "" };
    format!("{sign}{grouped}")
}

/// Formats a currency value with locale-appropriate formatting and a currency symbol.
///
/// # Example
/// ```
/// use aurora_i18n::locale::Locale;
/// use aurora_i18n::format::format_currency;
///
/// let en = Locale::new("en-US");
/// assert_eq!(format_currency(1234.50, "USD", &en), "USD 1,234.50");
/// ```
pub fn format_currency(value: f64, currency: &str, locale: &Locale) -> String {
    let formatted = format_number(value, 2, locale);
    format!("{currency} {formatted}")
}

/// Formats a date with locale-appropriate ordering and separator.
///
/// # Example
/// ```
/// use aurora_i18n::locale::Locale;
/// use aurora_i18n::format::format_date;
///
/// let en = Locale::new("en-US");
/// assert_eq!(format_date(2025, 3, 15, &en), "03/15/2025");
///
/// let de = Locale::new("de-DE");
/// assert_eq!(format_date(2025, 3, 15, &de), "15.03.2025");
///
/// let ja = Locale::new("ja-JP");
/// assert_eq!(format_date(2025, 3, 15, &ja), "2025/03/15");
/// ```
pub fn format_date(year: i32, month: u32, day: u32, locale: &Locale) -> String {
    let pat = pattern_for(locale.tag());
    let sep = pat.date_sep;
    match pat.date_order {
        0 => format!("{:02}{sep}{:02}{sep}{:04}", month, day, year), // MDY
        1 => format!("{:02}{sep}{:02}{sep}{:04}", day, month, year), // DMY
        _ => format!("{:04}{sep}{:02}{sep}{:02}", year, month, day), // YMD
    }
}

/// Adds thousand-grouping separators to a numeric string.
fn add_grouping(digits: &str, separator: char) -> String {
    let len = digits.len();
    if len <= 3 {
        return digits.to_string();
    }

    let mut result = String::with_capacity(len + len / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(separator);
        }
        result.push(ch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn en() -> Locale {
        Locale::new("en-US")
    }
    fn de() -> Locale {
        Locale::new("de-DE")
    }
    fn fr() -> Locale {
        Locale::new("fr-FR")
    }
    fn ja() -> Locale {
        Locale::new("ja-JP")
    }

    #[test]
    fn format_number_en() {
        assert_eq!(format_number(1234567.89, 2, &en()), "1,234,567.89");
    }

    #[test]
    fn format_number_de() {
        assert_eq!(format_number(1234567.89, 2, &de()), "1.234.567,89");
    }

    #[test]
    fn format_number_fr() {
        assert_eq!(format_number(1234567.89, 2, &fr()), "1 234 567,89");
    }

    #[test]
    fn format_number_negative() {
        assert_eq!(format_number(-42.5, 1, &en()), "-42.5");
    }

    #[test]
    fn format_number_no_decimals() {
        assert_eq!(format_number(1000.0, 0, &en()), "1,000");
    }

    #[test]
    fn format_integer_en() {
        assert_eq!(format_integer(1234567, &en()), "1,234,567");
    }

    #[test]
    fn format_integer_negative() {
        assert_eq!(format_integer(-999, &en()), "-999");
    }

    #[test]
    fn format_currency_en() {
        assert_eq!(format_currency(1234.50, "USD", &en()), "USD 1,234.50");
    }

    #[test]
    fn format_date_en() {
        assert_eq!(format_date(2025, 3, 15, &en()), "03/15/2025");
    }

    #[test]
    fn format_date_de() {
        assert_eq!(format_date(2025, 3, 15, &de()), "15.03.2025");
    }

    #[test]
    fn format_date_ja() {
        assert_eq!(format_date(2025, 3, 15, &ja()), "2025/03/15");
    }

    #[test]
    fn small_numbers_no_grouping() {
        assert_eq!(format_integer(42, &en()), "42");
        assert_eq!(format_integer(999, &en()), "999");
    }
}
