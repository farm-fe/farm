use std::fmt::{self, Write};

use codemap::Span;

use crate::{ast::CssStmt, error::SassResult, lexer::Lexer, parse::MediaQueryParser};

#[derive(Debug, Clone)]
pub(crate) struct MediaRule {
    pub query: Vec<MediaQuery>,
    pub body: Vec<CssStmt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MediaQuery {
    pub modifier: Option<String>,
    pub media_type: Option<String>,
    pub conditions: Vec<String>,
    pub conjunction: bool,
}

impl MediaQuery {
    pub fn matches_all_types(&self) -> bool {
        self.media_type.is_none()
            || self
                .media_type
                .as_ref()
                .map_or(false, |v| v.to_ascii_lowercase() == "all")
    }

    pub fn condition(
        conditions: Vec<String>,
        // default=true
        conjunction: bool,
    ) -> Self {
        Self {
            modifier: None,
            media_type: None,
            conditions,
            conjunction,
        }
    }

    pub fn media_type(
        media_type: Option<String>,
        modifier: Option<String>,
        conditions: Option<Vec<String>>,
    ) -> Self {
        Self {
            modifier,
            conjunction: true,
            media_type,
            conditions: conditions.unwrap_or_default(),
        }
    }

    pub fn parse_list(list: &str, span: Span) -> SassResult<Vec<Self>> {
        let toks = Lexer::new_from_string(list, span);

        MediaQueryParser::new(toks).parse()
    }

    #[allow(clippy::if_not_else)]
    pub(crate) fn merge(&self, other: &Self) -> MediaQueryMergeResult {
        if !self.conjunction || !other.conjunction {
            return MediaQueryMergeResult::Unrepresentable;
        }

        let this_modifier = self.modifier.as_ref().map(|m| m.to_ascii_lowercase());
        let this_type = self.media_type.as_ref().map(|m| m.to_ascii_lowercase());
        let other_modifier = other.modifier.as_ref().map(|m| m.to_ascii_lowercase());
        let other_type = other.media_type.as_ref().map(|m| m.to_ascii_lowercase());

        if this_type.is_none() && other_type.is_none() {
            return MediaQueryMergeResult::Success(Self::condition(
                self.conditions
                    .iter()
                    .chain(&other.conditions)
                    .cloned()
                    .collect(),
                true,
            ));
        }

        let modifier;
        let media_type;
        let conditions;

        if (this_modifier.as_deref() == Some("not")) != (other_modifier.as_deref() == Some("not")) {
            if this_modifier == other_modifier {
                let negative_conditions = if this_modifier.as_deref() == Some("not") {
                    &self.conditions
                } else {
                    &other.conditions
                };

                let positive_conditions = if this_modifier.as_deref() == Some("not") {
                    &other.conditions
                } else {
                    &self.conditions
                };

                // If the negative conditions are a subset of the positive conditions, the
                // query is empty. For example, `not screen and (color)` has no
                // intersection with `screen and (color) and (grid)`.
                //
                // However, `not screen and (color)` *does* intersect with `screen and
                // (grid)`, because it means `not (screen and (color))` and so it allows
                // a screen with no color but with a grid.

                if negative_conditions
                    .iter()
                    .all(|feat| positive_conditions.contains(feat))
                {
                    return MediaQueryMergeResult::Empty;
                }

                return MediaQueryMergeResult::Unrepresentable;
            } else if self.matches_all_types() || other.matches_all_types() {
                return MediaQueryMergeResult::Unrepresentable;
            }

            if this_modifier.as_deref() == Some("not") {
                modifier = &other_modifier;
                media_type = &other_type;
                conditions = other.conditions.clone();
            } else {
                modifier = &this_modifier;
                media_type = &this_type;
                conditions = self.conditions.clone();
            }
        } else if this_modifier.as_deref() == Some("not") {
            debug_assert_eq!(other_modifier.as_deref(), Some("not"));

            // CSS has no way of representing "neither screen nor print".
            if this_type != other_type {
                return MediaQueryMergeResult::Unrepresentable;
            }

            let more_conditions = if self.conditions.len() > other.conditions.len() {
                &self.conditions
            } else {
                &other.conditions
            };

            let fewer_conditions = if self.conditions.len() > other.conditions.len() {
                &other.conditions
            } else {
                &self.conditions
            };

            // If one set of conditions is a superset of the other, use those conditions
            // because they're strictly narrower.
            if fewer_conditions
                .iter()
                .all(|feat| more_conditions.contains(feat))
            {
                modifier = &this_modifier;
                media_type = &this_type;
                conditions = more_conditions.clone();
            } else {
                // Otherwise, there's no way to represent the intersection.
                return MediaQueryMergeResult::Unrepresentable;
            }
        } else if self.matches_all_types() {
            modifier = &other_modifier;

            // Omit the type if either input query did, since that indicates that they
            // aren't targeting a browser that requires "all and".
            media_type = if other.matches_all_types() && this_type.is_none() {
                &None
            } else {
                &other_type
            };

            conditions = self
                .conditions
                .iter()
                .chain(&other.conditions)
                .cloned()
                .collect();
        } else if other.matches_all_types() {
            modifier = &this_modifier;
            media_type = &this_type;
            conditions = self
                .conditions
                .iter()
                .chain(&other.conditions)
                .cloned()
                .collect();
        } else if this_type != other_type {
            return MediaQueryMergeResult::Empty;
        } else {
            if this_modifier.is_some() {
                modifier = &this_modifier;
            } else {
                modifier = &other_modifier;
            }

            media_type = &this_type;
            conditions = self
                .conditions
                .iter()
                .chain(&other.conditions)
                .cloned()
                .collect();
        }

        MediaQueryMergeResult::Success(MediaQuery {
            media_type: if media_type == &this_type {
                self.media_type.clone()
            } else {
                other.media_type.clone()
            },
            modifier: if modifier == &this_modifier {
                self.modifier.clone()
            } else {
                other.modifier.clone()
            },
            conditions,
            conjunction: true,
        })
    }
}

impl fmt::Display for MediaQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(modifier) = &self.modifier {
            f.write_str(modifier)?;
            f.write_char(' ')?;
        }

        if let Some(media_type) = &self.media_type {
            f.write_str(media_type)?;
            if !&self.conditions.is_empty() {
                f.write_str(" and ")?;
            }
        }

        f.write_str(&self.conditions.join(" and "))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum MediaQueryMergeResult {
    Empty,
    Unrepresentable,
    Success(MediaQuery),
}
