use codemap::Span;

use crate::ast::CssMediaQuery;

use super::{ComplexSelector, SimpleSelector};

#[derive(Clone, Debug)]
pub(crate) struct Extension {
    /// The selector in which the `@extend` appeared.
    pub extender: ComplexSelector,

    /// The selector that's being extended.
    ///
    /// `None` for one-off extensions.
    pub target: Option<SimpleSelector>,

    /// The minimum specificity required for any selector generated from this
    /// extender.
    pub specificity: i32,

    /// Whether this extension is optional.
    pub is_optional: bool,

    /// Whether this is a one-off extender representing a selector that was
    /// originally in the document, rather than one defined with `@extend`.
    pub is_original: bool,

    /// The media query context to which this extend is restricted, or `None` if
    /// it can apply within any context.
    pub media_context: Option<Vec<CssMediaQuery>>,

    /// The span in which `extender` was defined.
    pub span: Span,

    #[allow(dead_code)]
    pub left: Option<Box<Extension>>,

    #[allow(dead_code)]
    pub right: Option<Box<Extension>>,
}

impl Extension {
    pub fn one_off(
        extender: ComplexSelector,
        specificity: Option<i32>,
        is_original: bool,
        span: Span,
    ) -> Self {
        Self {
            specificity: specificity.unwrap_or_else(|| extender.max_specificity()),
            extender,
            target: None,
            span,
            is_optional: true,
            is_original,
            media_context: None,
            left: None,
            right: None,
        }
    }

    /// Asserts that the `media_context` for a selector is compatible with the
    /// query context for this extender.
    // todo: this should return a `Result`. it currently does not because the cascade effect
    // from this returning a `Result` will make some code returning `Option`s much uglier (we can't
    // use `?` to return both `Option` and `Result` from the same function)
    #[allow(clippy::needless_return)]
    pub fn assert_compatible_media_context(&self, media_context: &Option<Vec<CssMediaQuery>>) {
        if &self.media_context == media_context {
            return;
        }

        // Err(("You may not @extend selectors across media queries.", self.span).into())
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_extender(mut self, extender: ComplexSelector) -> Self {
        self.extender = extender;
        self
    }
}
