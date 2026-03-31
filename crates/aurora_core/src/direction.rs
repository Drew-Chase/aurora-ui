/// The inline text direction, controlling layout flow along the horizontal axis.
///
/// Determines whether [`Row`] children flow left-to-right or right-to-left,
/// how `Align::Start`/`Align::End` resolve on the cross axis of [`Column`],
/// and which physical edge [`Edges::inline_start`] maps to.
///
/// # Examples
///
/// ```
/// use aurora_core::direction::TextDirection;
///
/// let dir = TextDirection::Rtl;
/// assert!(dir.is_rtl());
/// assert!(!dir.is_ltr());
///
/// let default = TextDirection::default();
/// assert!(default.is_ltr());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextDirection {
    /// Left-to-right (Latin, Cyrillic, CJK, etc.).
    #[default]
    Ltr,
    /// Right-to-left (Arabic, Hebrew, Urdu, etc.).
    Rtl,
}

impl TextDirection {
    /// Returns `true` for right-to-left direction.
    pub fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }

    /// Returns `true` for left-to-right direction.
    pub fn is_ltr(self) -> bool {
        matches!(self, Self::Ltr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_ltr() {
        assert_eq!(TextDirection::default(), TextDirection::Ltr);
    }

    #[test]
    fn is_rtl() {
        assert!(TextDirection::Rtl.is_rtl());
        assert!(!TextDirection::Ltr.is_rtl());
    }

    #[test]
    fn is_ltr() {
        assert!(TextDirection::Ltr.is_ltr());
        assert!(!TextDirection::Rtl.is_ltr());
    }
}
