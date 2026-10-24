use crate::error::SassResult;

use super::Extension;

/// An `Extension` created by merging two `Extension`s with the same extender
/// and target.
///
/// This is used when multiple mandatory extensions exist to ensure that both of
/// them are marked as resolved.
pub(super) struct MergedExtension;

impl MergedExtension {
    /// Returns an extension that combines `left` and `right`.
    ///
    /// Returns an `Err` if `left` and `right` have incompatible media
    /// contexts.
    ///
    /// Returns an `Err` if `left` and `right` don't have the same
    /// extender and target.
    pub fn merge(left: Extension, right: Extension) -> SassResult<Extension> {
        if left.extender != right.extender || left.target != right.target {
            return Err((
                format!(
                    "{} and {} aren't the same extension.",
                    left.extender, right.extender
                ),
                left.span.merge(right.span),
            )
                .into());
        }

        if left.media_context.is_some()
            && right.media_context.is_some()
            && left.media_context != right.media_context
        {
            return Err((
                "You may not @extend the same selector from within different media queries.",
                right.span,
            )
                .into());
        }

        if right.is_optional && right.media_context.is_none() {
            return Ok(left);
        }

        if left.is_optional && left.media_context.is_none() {
            return Ok(right);
        }

        Ok(MergedExtension::into_extension(left, right))
    }

    fn into_extension(left: Extension, right: Extension) -> Extension {
        Extension {
            extender: left.extender,
            target: left.target,
            span: left.span,
            media_context: match left.media_context {
                Some(v) => Some(v),
                None => right.media_context,
            },
            specificity: left.specificity,
            is_optional: true,
            is_original: false,
            left: None,
            right: None,
        }
    }
}
