use std::{
    collections::VecDeque,
    fmt::{self, Write},
    hash::{Hash, Hasher},
    mem,
};

use codemap::Span;

use super::{unify_complex, ComplexSelector, ComplexSelectorComponent};

use crate::{
    common::{Brackets, ListSeparator, QuoteKind},
    error::SassResult,
    value::Value,
};

/// A selector list.
///
/// A selector list is composed of `ComplexSelector`s. It matches an element
/// that matches any of the component selectors.
#[derive(Clone, Debug)]
pub(crate) struct SelectorList {
    /// The components of this selector.
    ///
    /// This is never empty.
    pub components: Vec<ComplexSelector>,
    pub span: Span,
}

impl PartialEq for SelectorList {
    fn eq(&self, other: &SelectorList) -> bool {
        self.components == other.components
    }
}

impl Eq for SelectorList {}

impl Hash for SelectorList {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.components.hash(state);
    }
}

impl fmt::Display for SelectorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let complexes = self.components.iter().filter(|c| !c.is_invisible());

        let mut first = true;

        for complex in complexes {
            if first {
                first = false;
            } else {
                f.write_char(',')?;
                if complex.line_break {
                    f.write_char('\n')?;
                } else {
                    f.write_char(' ')?;
                }
            }
            write!(f, "{}", complex)?;
        }
        Ok(())
    }
}

impl SelectorList {
    pub fn is_invisible(&self) -> bool {
        self.components.iter().all(ComplexSelector::is_invisible)
    }

    pub fn contains_parent_selector(&self) -> bool {
        self.components
            .iter()
            .any(ComplexSelector::contains_parent_selector)
    }

    pub const fn new(span: Span) -> Self {
        Self {
            components: Vec::new(),
            span,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Returns a `SassScript` list that represents this selector.
    ///
    /// This has the same format as a list returned by `selector-parse()`.
    pub fn to_sass_list(self) -> Value {
        Value::List(
            self.components
                .into_iter()
                .map(|complex| {
                    Value::List(
                        complex
                            .components
                            .into_iter()
                            .map(|complex_component| {
                                Value::String(complex_component.to_string(), QuoteKind::None)
                            })
                            .collect(),
                        ListSeparator::Space,
                        Brackets::None,
                    )
                })
                .collect(),
            ListSeparator::Comma,
            Brackets::None,
        )
    }

    /// Returns a `SelectorList` that matches only elements that are matched by
    /// both this and `other`.
    ///
    /// If no such list can be produced, returns `None`.
    pub fn unify(self, other: &Self) -> Option<Self> {
        let contents: Vec<ComplexSelector> = self
            .components
            .into_iter()
            .flat_map(|c1| {
                other.clone().components.into_iter().flat_map(move |c2| {
                    let unified: Option<Vec<Vec<ComplexSelectorComponent>>> =
                        unify_complex(vec![c1.components.clone(), c2.components]);
                    if let Some(u) = unified {
                        u.into_iter()
                            .map(|c| ComplexSelector::new(c, false))
                            .collect()
                    } else {
                        Vec::new()
                    }
                })
            })
            .collect();

        if contents.is_empty() {
            return None;
        }

        Some(Self {
            components: contents,
            span: self.span.merge(other.span),
        })
    }

    /// Returns a new list with all `SimpleSelector::Parent`s replaced with `parent`.
    ///
    /// If `implicit_parent` is true, this treats `ComplexSelector`s that don't
    /// contain an explicit `SimpleSelector::Parent` as though they began with one.
    ///
    /// The given `parent` may be `None`, indicating that this has no parents. If
    /// so, this list is returned as-is if it doesn't contain any explicit
    /// `SimpleSelector::Parent`s. If it does, this returns a `SassError`.
    pub fn resolve_parent_selectors(
        self,
        parent: Option<Self>,
        implicit_parent: bool,
    ) -> SassResult<Self> {
        let parent = match parent {
            Some(p) => p,
            None => {
                if !self.contains_parent_selector() {
                    return Ok(self);
                }
                return Err((
                    "Top-level selectors may not contain the parent selector \"&\".",
                    self.span,
                )
                    .into());
            }
        };

        Ok(Self {
            components: flatten_vertically(
                self.components
                    .into_iter()
                    .map(|complex| {
                        if !complex.contains_parent_selector() {
                            if !implicit_parent {
                                return Ok(vec![complex]);
                            }
                            return Ok(parent
                                .clone()
                                .components
                                .into_iter()
                                .map(move |parent_complex| {
                                    let mut components = parent_complex.components;
                                    components.append(&mut complex.components.clone());
                                    ComplexSelector::new(
                                        components,
                                        complex.line_break || parent_complex.line_break,
                                    )
                                })
                                .collect());
                        }

                        let mut new_complexes: Vec<Vec<ComplexSelectorComponent>> =
                            vec![Vec::new()];
                        let mut line_breaks = vec![false];

                        for component in complex.components {
                            if component.is_compound() {
                                let resolved = match component
                                    .clone()
                                    .resolve_parent_selectors(self.span, parent.clone())?
                                {
                                    Some(r) => r,
                                    None => {
                                        for new_complex in &mut new_complexes {
                                            new_complex.push(component.clone());
                                        }
                                        continue;
                                    }
                                };

                                let previous_complexes = mem::take(&mut new_complexes);
                                let previous_line_breaks = mem::take(&mut line_breaks);

                                for (i, new_complex) in previous_complexes.into_iter().enumerate() {
                                    // todo: use .get(i)
                                    let line_break = previous_line_breaks[i];
                                    for mut resolved_complex in resolved.clone() {
                                        let mut new_this_complex = new_complex.clone();
                                        new_this_complex.append(&mut resolved_complex.components);
                                        new_complexes.push(mem::take(&mut new_this_complex));
                                        line_breaks.push(line_break || resolved_complex.line_break);
                                    }
                                }
                            } else {
                                for new_complex in &mut new_complexes {
                                    new_complex.push(component.clone());
                                }
                            }
                        }

                        let mut i = 0;
                        Ok(new_complexes
                            .into_iter()
                            .map(|new_complex| {
                                i += 1;
                                ComplexSelector::new(new_complex, line_breaks[i - 1])
                            })
                            .collect())
                    })
                    .collect::<SassResult<Vec<Vec<ComplexSelector>>>>()?,
            ),
            span: self.span,
        })
    }

    pub fn is_superselector(&self, other: &Self) -> bool {
        other.components.iter().all(|complex1| {
            self.components
                .iter()
                .any(|complex2| complex2.is_super_selector(complex1))
        })
    }
}

fn flatten_vertically<A: std::fmt::Debug>(iterable: Vec<Vec<A>>) -> Vec<A> {
    let mut queues: Vec<VecDeque<A>> = iterable.into_iter().map(VecDeque::from).collect();

    let mut result = Vec::new();

    while !queues.is_empty() {
        for queue in &mut queues {
            if queue.is_empty() {
                continue;
            }
            result.push(queue.pop_front().unwrap());
        }

        queues.retain(|queue| !queue.is_empty());
    }

    result
}
