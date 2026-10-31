use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use codemap::Span;

use indexmap::IndexMap;

use crate::{ast::CssMediaQuery, error::SassResult};

use super::{
    ComplexSelector, ComplexSelectorComponent, ComplexSelectorHashSet, CompoundSelector, Pseudo,
    SelectorList, SimpleSelector,
};

pub(crate) use extended_selector::ExtendedSelector;
use extended_selector::SelectorHashSet;
use extension::Extension;
pub(crate) use functions::unify_complex;
use functions::{paths, weave};
use merged::MergedExtension;
pub(crate) use rule::ExtendRule;

mod extended_selector;
mod extension;
mod functions;
mod merged;
mod rule;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// Different modes in which extension can run.
enum ExtendMode {
    /// Normal mode, used with the `@extend` rule.
    ///
    /// This preserves existing selectors and extends each target individually.
    Normal,

    /// Replace mode, used by the `selector-replace()` function.
    ///
    /// This replaces existing selectors and requires every target to match to
    /// extend a given compound selector.
    Replace,

    /// All-targets mode, used by the `selector-extend()` function.
    ///
    /// This preserves existing selectors but requires every target to match to
    /// extend a given compound selector.
    AllTargets,
}

impl Default for ExtendMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ExtensionStore {
    /// A map from all simple selectors in the stylesheet to the selector lists
    /// that contain them.
    ///
    /// This is used to find which selectors an `@extend` applies to and adjust
    /// them.
    selectors: HashMap<SimpleSelector, SelectorHashSet>,

    /// A map from all extended simple selectors to the sources of those
    /// extensions.
    extensions: HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>,

    /// A map from all simple selectors in extenders to the extensions that those
    /// extenders define.
    extensions_by_extender: HashMap<SimpleSelector, Vec<Extension>>,

    /// A map from CSS selectors to the media query contexts they're defined in.
    ///
    /// This tracks the contexts in which each selector's style rule is defined.
    /// If a rule is defined at the top level, it doesn't have an entry.
    media_contexts: HashMap<SelectorList, Vec<CssMediaQuery>>,

    /// A map from `SimpleSelector`s to the specificity of their source
    /// selectors.
    ///
    /// This tracks the maximum specificity of the `ComplexSelector` that
    /// originally contained each `SimpleSelector`. This allows us to ensure that
    /// we don't trim any selectors that need to exist to satisfy the [second law
    /// of extend][].
    ///
    /// [second law of extend]: https://github.com/sass/sass/issues/324#issuecomment-4607184
    source_specificity: HashMap<SimpleSelector, i32>,

    /// A set of `ComplexSelector`s that were originally part of
    /// their component `SelectorList`s, as opposed to being added by `@extend`.
    ///
    /// This allows us to ensure that we don't trim any selectors that need to
    /// exist to satisfy the [first law of extend][].
    ///
    /// [first law of extend]: https://github.com/sass/sass/issues/324#issuecomment-4607184
    originals: ComplexSelectorHashSet,

    /// The mode that controls this extender's behavior.
    mode: ExtendMode,

    span: Span,
}

impl ExtensionStore {
    /// An `Extender` that contains no extensions and can have no extensions added.
    // TODO: empty extender
    #[allow(dead_code)]
    const EMPTY: () = ();

    pub fn extend(
        selector: SelectorList,
        source: SelectorList,
        targets: SelectorList,
        span: Span,
    ) -> SassResult<SelectorList> {
        Self::extend_or_replace(selector, source, targets, ExtendMode::AllTargets, span)
    }

    pub fn new(span: Span) -> Self {
        Self {
            selectors: HashMap::new(),
            extensions: HashMap::new(),
            extensions_by_extender: HashMap::new(),
            media_contexts: HashMap::new(),
            source_specificity: HashMap::new(),
            originals: ComplexSelectorHashSet::new(),
            mode: ExtendMode::Normal,
            span,
        }
    }

    pub fn replace(
        selector: SelectorList,
        source: SelectorList,
        targets: SelectorList,
        span: Span,
    ) -> SassResult<SelectorList> {
        Self::extend_or_replace(selector, source, targets, ExtendMode::Replace, span)
    }

    fn extend_or_replace(
        selector: SelectorList,
        source: SelectorList,
        targets: SelectorList,
        mode: ExtendMode,
        span: Span,
    ) -> SassResult<SelectorList> {
        let extenders: IndexMap<ComplexSelector, Extension> = source
            .components
            .into_iter()
            .map(|complex| {
                (
                    complex.clone(),
                    Extension::one_off(complex, None, false, span),
                )
            })
            .collect();

        let compound_targets = targets
            .components
            .into_iter()
            .map(|complex| {
                if complex.components.len() == 1 {
                    Ok(complex.components.first().unwrap().as_compound().clone())
                } else {
                    Err((format!("Can't extend complex selector {}.", complex), span).into())
                }
            })
            .collect::<SassResult<Vec<CompoundSelector>>>()?;

        let extensions: HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>> =
            compound_targets
                .into_iter()
                .flat_map(|compound| {
                    compound
                        .components
                        .into_iter()
                        .map(|simple| (simple, extenders.clone()))
                })
                .collect();

        let mut extender = ExtensionStore::with_mode(mode, span);

        if !selector.is_invisible() {
            extender.originals.extend(selector.components.iter());
        }

        Ok(extender.extend_list(selector, Some(&extensions), &None))
    }

    fn with_mode(mode: ExtendMode, span: Span) -> Self {
        Self {
            mode,
            ..ExtensionStore::new(span)
        }
    }

    /// Extends `list` using `extensions`.
    fn extend_list(
        &mut self,
        list: SelectorList,
        extensions: Option<&HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>>,
        media_query_context: &Option<Vec<CssMediaQuery>>,
    ) -> SelectorList {
        // This could be written more simply using Vec<Vec<T>>, but we want to avoid
        // any allocations in the common case where no extends apply.
        let mut extended: Option<Vec<ComplexSelector>> = None;
        for (i, complex) in list.components.iter().enumerate() {
            if let Some(result) =
                self.extend_complex(complex.clone(), extensions, media_query_context)
            {
                if extended.is_none() {
                    extended = Some(if i == 0 {
                        Vec::new()
                    } else {
                        list.components[0..i].to_vec()
                    });
                }
                match extended.as_mut() {
                    Some(v) => v.extend(result.into_iter()),
                    None => unreachable!(),
                }
            } else if let Some(extended) = extended.as_mut() {
                extended.push(complex.clone());
            }
        }

        let extended = match extended {
            Some(v) => v,
            None => return list,
        };

        SelectorList {
            components: self.trim(extended, &|complex| self.originals.contains(complex)),
            span: self.span,
        }
    }

    /// Extends `complex` using `extensions`, and returns the contents of a
    /// `SelectorList`.
    fn extend_complex(
        &mut self,
        complex: ComplexSelector,
        extensions: Option<&HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>>,
        media_query_context: &Option<Vec<CssMediaQuery>>,
    ) -> Option<Vec<ComplexSelector>> {
        // The complex selectors that each compound selector in `complex.components`
        // can expand to.
        //
        // For example, given
        //
        //     .a .b {...}
        //     .x .y {@extend .b}
        //
        // this will contain
        //
        //     [
        //       [.a],
        //       [.b, .x .y]
        //     ]
        //
        // This could be written more simply using `Vec::into_iter::map`, but we want to avoid
        // any allocations in the common case where no extends apply.
        let mut extended_not_expanded: Option<Vec<Vec<ComplexSelector>>> = None;

        let complex_has_line_break = complex.line_break;

        let is_original = self.originals.contains(&complex);

        for (i, component) in complex.components.iter().enumerate() {
            if let ComplexSelectorComponent::Compound(component) = component {
                if let Some(extended) =
                    self.extend_compound(component, extensions, media_query_context, is_original)
                {
                    if extended_not_expanded.is_none() {
                        extended_not_expanded = Some(
                            complex
                                .components
                                .clone()
                                .into_iter()
                                .take(i)
                                .map(|component| {
                                    vec![ComplexSelector::new(vec![component], complex.line_break)]
                                })
                                .collect(),
                        );
                    }
                    match extended_not_expanded.as_mut() {
                        Some(v) => v.push(extended),
                        None => unreachable!(),
                    }
                } else {
                    match extended_not_expanded.as_mut() {
                        Some(v) => v.push(vec![ComplexSelector::new(
                            vec![ComplexSelectorComponent::Compound(component.clone())],
                            false,
                        )]),
                        None => {}
                    }
                }
            } else if component.is_combinator() {
                match extended_not_expanded.as_mut() {
                    Some(v) => v.push(vec![ComplexSelector::new(vec![component.clone()], false)]),
                    None => {}
                }
            }
        }

        let extended_not_expanded = extended_not_expanded?;

        let mut first = true;

        Some(
            paths(extended_not_expanded)
                .into_iter()
                .flat_map(move |path| {
                    weave(
                        path.clone()
                            .into_iter()
                            .map(move |complex| complex.components)
                            .collect(),
                    )
                    .into_iter()
                    .map(|components| {
                        let output_complex = ComplexSelector::new(
                            components,
                            complex_has_line_break
                                || path.iter().any(|input_complex| input_complex.line_break),
                        );

                        // Make sure that copies of `complex` retain their status as "original"
                        // selectors. This includes selectors that are modified because a :not()
                        // was extended into.
                        if first && self.originals.contains(&complex) {
                            self.originals.insert(&output_complex);
                        }
                        first = false;

                        output_complex
                    })
                    .collect::<Vec<ComplexSelector>>()
                })
                .collect(),
        )
    }

    /// Extends `compound` using `extensions`, and returns the contents of a
    /// `SelectorList`.
    ///
    /// The `in_original` parameter indicates whether this is in an original
    /// complex selector, meaning that `compound` should not be trimmed out.
    fn extend_compound(
        &mut self,
        compound: &CompoundSelector,
        extensions: Option<&HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>>,
        media_query_context: &Option<Vec<CssMediaQuery>>,
        in_original: bool,
    ) -> Option<Vec<ComplexSelector>> {
        // If there's more than one target and they all need to match, we track
        // which targets are actually extended.
        let mut targets_used: HashSet<SimpleSelector> = HashSet::new();

        let mut options: Option<Vec<Vec<Extension>>> = None;

        for i in 0..compound.components.len() {
            let simple = compound.components.get(i).cloned().unwrap();

            match self.extend_simple(
                simple.clone(),
                extensions,
                media_query_context,
                &mut targets_used,
            ) {
                Some(extended) => {
                    if options.is_none() {
                        let mut new_options = Vec::new();
                        if i != 0 {
                            new_options.push(vec![
                                self.extension_for_compound(compound.components[..i].to_vec())
                            ]);
                        }
                        options.replace(new_options);
                    }

                    match options.as_mut() {
                        Some(v) => v.extend(extended.into_iter()),
                        None => unreachable!(),
                    }
                }
                None => match options.as_mut() {
                    Some(v) => v.push(vec![self.extension_for_simple(simple)]),
                    None => {}
                },
            }
        }

        let options = options?;

        // If `self.mode` isn't `ExtendMode::Normal` and we didn't use all the targets in
        // `extensions`, extension fails for `compound`.
        // todo: test for `extensions.len() > 2`. may cause issues
        if !targets_used.is_empty()
            && targets_used.len() != extensions.map_or(self.extensions.len(), HashMap::len)
            && self.mode != ExtendMode::Normal
        {
            return None;
        }

        // Optimize for the simple case of a single simple selector that doesn't
        // need any unification.
        if options.len() == 1 {
            return Some(
                options
                    .first()?
                    .clone()
                    .into_iter()
                    .map(|state| {
                        state.assert_compatible_media_context(media_query_context);
                        state.extender
                    })
                    .collect(),
            );
        }

        // Find all paths through `options`. In this case, each path represents a
        // different unification of the base selector. For example, if we have:
        //
        //     .a.b {...}
        //     .w .x {@extend .a}
        //     .y .z {@extend .b}
        //
        // then `options` is `[[.a, .w .x], [.b, .y .z]]` and `paths(options)` is
        //
        //     [
        //       [.a, .b],
        //       [.a, .y .z],
        //       [.w .x, .b],
        //       [.w .x, .y .z]
        //     ]
        //
        // We then unify each path to get a list of complex selectors:
        //
        //     [
        //       [.a.b],
        //       [.y .a.z],
        //       [.w .x.b],
        //       [.w .y .x.z, .y .w .x.z]
        //     ]
        let mut first = self.mode != ExtendMode::Replace;

        let unified_paths = paths(options).into_iter().map(|path| {
            let complexes: Vec<Vec<ComplexSelectorComponent>> = if first {
                // The first path is always the original selector. We can't just
                // return `compound` directly because pseudo selectors may be
                // modified, but we don't have to do any unification.
                first = false;

                vec![vec![ComplexSelectorComponent::Compound(CompoundSelector {
                    components: path
                        .clone()
                        .into_iter()
                        .flat_map(|state| {
                            debug_assert!(state.extender.components.len() == 1);
                            match state.extender.components.last().cloned() {
                                Some(ComplexSelectorComponent::Compound(c)) => c.components,
                                Some(..) | None => unreachable!(),
                            }
                        })
                        .collect(),
                })]]
            } else {
                let mut to_unify: VecDeque<Vec<ComplexSelectorComponent>> = VecDeque::new();
                let mut originals: Vec<SimpleSelector> = Vec::new();

                for state in path.clone() {
                    if state.is_original {
                        originals.extend(match state.extender.components.last().cloned() {
                            Some(ComplexSelectorComponent::Compound(c)) => c.components,
                            Some(..) | None => unreachable!(),
                        });
                    } else {
                        to_unify.push_back(state.extender.components.clone());
                    }
                }
                if !originals.is_empty() {
                    to_unify.push_front(vec![ComplexSelectorComponent::Compound(
                        CompoundSelector {
                            components: originals,
                        },
                    )]);
                }

                unify_complex(Vec::from(to_unify))?
            };

            let mut line_break = false;

            for state in path {
                state.assert_compatible_media_context(media_query_context);
                line_break = line_break || state.extender.line_break;
            }

            Some(
                complexes
                    .into_iter()
                    .map(|components| ComplexSelector::new(components, line_break))
                    .collect::<Vec<ComplexSelector>>(),
            )
        });

        let unified_paths: Vec<ComplexSelector> = unified_paths.flatten().flatten().collect();

        Some(if in_original && self.mode != ExtendMode::Replace {
            let original = unified_paths.first().cloned();
            self.trim(unified_paths, &|complex| Some(complex) == original.as_ref())
        } else {
            self.trim(unified_paths, &|_| false)
        })
    }

    fn extend_simple(
        &mut self,
        simple: SimpleSelector,
        extensions: Option<&HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>>,
        media_query_context: &Option<Vec<CssMediaQuery>>,
        targets_used: &mut HashSet<SimpleSelector>,
    ) -> Option<Vec<Vec<Extension>>> {
        if let SimpleSelector::Pseudo(Pseudo {
            selector: Some(..), ..
        }) = &simple
        {
            let simple = if let SimpleSelector::Pseudo(pseudo) = simple.clone() {
                pseudo
            } else {
                unreachable!()
            };
            if let Some(extended) = self.extend_pseudo(simple, extensions, media_query_context) {
                return Some(
                    extended
                        .into_iter()
                        .map(move |pseudo| {
                            self.without_pseudo(
                                SimpleSelector::Pseudo(pseudo.clone()),
                                extensions,
                                targets_used,
                                self.mode,
                            )
                            .unwrap_or_else(|| {
                                vec![self.extension_for_simple(SimpleSelector::Pseudo(pseudo))]
                            })
                        })
                        .collect(),
                );
            }
        }

        self.without_pseudo(simple, extensions, targets_used, self.mode)
            .map(|v| vec![v])
    }

    /// Extends `pseudo` using `extensions`, and returns a list of resulting
    /// pseudo selectors.
    fn extend_pseudo(
        &mut self,
        pseudo: Pseudo,
        extensions: Option<&HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>>,
        media_query_context: &Option<Vec<CssMediaQuery>>,
    ) -> Option<Vec<Pseudo>> {
        let extended = self.extend_list(
            pseudo
                .selector
                .as_deref()
                .cloned()
                .unwrap_or_else(|| SelectorList::new(self.span)),
            extensions,
            media_query_context,
        );
        /*todo: identical(extended, pseudo.selector)*/
        if Some(&extended) == pseudo.selector.as_deref() {
            return None;
        }

        // For `:not()`, we usually want to get rid of any complex selectors because
        // that will cause the selector to fail to parse on all browsers at time of
        // writing. We can keep them if either the original selector had a complex
        // selector, or the result of extending has only complex selectors, because
        // either way we aren't breaking anything that isn't already broken.
        let mut complexes = if pseudo.normalized_name() == "not"
            && !pseudo
                .selector
                .clone()
                .unwrap()
                .components
                .iter()
                .any(|complex| complex.components.len() > 1)
            && extended
                .components
                .iter()
                .any(|complex| complex.components.len() == 1)
        {
            extended
                .components
                .into_iter()
                .filter(|complex| complex.components.len() <= 1)
                .collect()
        } else {
            extended.components
        };

        complexes = complexes
            .into_iter()
            .flat_map(|complex| {
                if complex.components.len() != 1 {
                    return vec![complex];
                }
                let compound = match complex.components.first() {
                    Some(ComplexSelectorComponent::Compound(c)) => c,
                    Some(..) | None => return vec![complex],
                };
                if compound.components.len() != 1 {
                    return vec![complex];
                }
                if !compound.components.first().unwrap().is_pseudo() {
                    return vec![complex];
                }
                let inner_pseudo = match compound.components.first() {
                    Some(SimpleSelector::Pseudo(pseudo)) => pseudo,
                    Some(..) | None => return vec![complex],
                };
                if inner_pseudo.selector.is_none() {
                    return vec![complex];
                }

                match pseudo.normalized_name() {
                    "not" => {
                        // In theory, if there's a `:not` nested within another `:not`, the
                        // inner `:not`'s contents should be unified with the return value.
                        // For example, if `:not(.foo)` extends `.bar`, `:not(.bar)` should
                        // become `.foo:not(.bar)`. However, this is a narrow edge case and
                        // supporting it properly would make this code and the code calling it
                        // a lot more complicated, so it's not supported for now.
                        let inner_pseudo_normalized = inner_pseudo.normalized_name();
                        if ["matches", "is", "where"].contains(&inner_pseudo_normalized) {
                            inner_pseudo.selector.clone().unwrap().components
                        } else {
                            Vec::new()
                        }
                    }
                    "matches" | "where" | "is" | "any" | "current" | "nth-child"
                    | "nth-last-child" => {
                        // As above, we could theoretically support :not within :matches, but
                        // doing so would require this method and its callers to handle much
                        // more complex cases that likely aren't worth the pain.
                        if inner_pseudo.name != pseudo.name
                            || inner_pseudo.argument != pseudo.argument
                        {
                            Vec::new()
                        } else {
                            inner_pseudo.selector.clone().unwrap().components
                        }
                    }
                    "has" | "host" | "host-context" | "slotted" => {
                        // We can't expand nested selectors here, because each layer adds an
                        // additional layer of semantics. For example, `:has(:has(img))`
                        // doesn't match `<div><img></div>` but `:has(img)` does.
                        vec![complex]
                    }
                    _ => Vec::new(),
                }
            })
            .collect();

        // Older browsers support `:not`, but only with a single complex selector.
        // In order to support those browsers, we break up the contents of a `:not`
        // unless it originally contained a selector list.
        if pseudo.normalized_name() == "not"
            && pseudo.selector.clone().unwrap().components.len() == 1
        {
            let result = complexes
                .into_iter()
                .map(|complex| {
                    pseudo.clone().with_selector(Some(Box::new(SelectorList {
                        components: vec![complex],
                        span: self.span,
                    })))
                })
                .collect::<Vec<Pseudo>>();
            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        } else {
            Some(vec![pseudo.with_selector(Some(Box::new(SelectorList {
                components: complexes,
                span: self.span,
            })))])
        }
    }

    /// Extends `simple` without extending the contents of any selector pseudos
    /// it contains.
    fn without_pseudo(
        &self,
        simple: SimpleSelector,
        extensions: Option<&HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>>,
        targets_used: &mut HashSet<SimpleSelector>,
        mode: ExtendMode,
    ) -> Option<Vec<Extension>> {
        let extenders = extensions.unwrap_or(&self.extensions).get(&simple)?;

        targets_used.insert(simple.clone());

        if mode == ExtendMode::Replace {
            return Some(extenders.values().cloned().collect());
        }

        let mut tmp = vec![self.extension_for_simple(simple)];
        tmp.reserve(extenders.len());
        tmp.extend(extenders.values().cloned());

        Some(tmp)
    }

    /// Returns a one-off `Extension` whose extender is composed solely of
    /// `simple`.
    fn extension_for_simple(&self, simple: SimpleSelector) -> Extension {
        let specificity = Some(*self.source_specificity.get(&simple).unwrap_or(&0_i32));
        Extension::one_off(
            ComplexSelector::new(
                vec![ComplexSelectorComponent::Compound(CompoundSelector {
                    components: vec![simple],
                })],
                false,
            ),
            specificity,
            true,
            self.span,
        )
    }

    /// Returns a one-off `Extension` whose extender is composed solely of a
    /// compound selector containing `simples`.
    fn extension_for_compound(&self, simples: Vec<SimpleSelector>) -> Extension {
        let compound = CompoundSelector {
            components: simples,
        };
        let specificity = Some(self.source_specificity_for(&compound));
        Extension::one_off(
            ComplexSelector::new(vec![ComplexSelectorComponent::Compound(compound)], false),
            specificity,
            true,
            self.span,
        )
    }

    /// Returns the maximum specificity for sources that went into producing
    /// `compound`.
    fn source_specificity_for(&self, compound: &CompoundSelector) -> i32 {
        let mut specificity = 0;
        for simple in &compound.components {
            specificity = specificity.max(*self.source_specificity.get(simple).unwrap_or(&0));
        }
        specificity
    }

    /// Removes elements from `selectors` if they're subselectors of other
    /// elements.
    ///
    /// The `is_original` callback indicates which selectors are original to the
    /// document, and thus should never be trimmed.
    fn trim(
        &self,
        selectors: Vec<ComplexSelector>,
        is_original: &dyn Fn(&ComplexSelector) -> bool,
    ) -> Vec<ComplexSelector> {
        // Avoid truly horrific quadratic behavior.
        //
        // TODO(nweiz): I think there may be a way to get perfect trimming without
        // going quadratic by building some sort of trie-like data structure that
        // can be used to look up superselectors.
        if selectors.len() > 100 {
            return selectors;
        }

        // This is n² on the sequences, but only comparing between separate
        // sequences should limit the quadratic behavior. We iterate from last to
        // first and reverse the result so that, if two selectors are identical, we
        // keep the first one.
        let mut result: VecDeque<ComplexSelector> = VecDeque::new();
        let mut num_originals = 0;

        // :outer
        for i in (0..=(selectors.len().saturating_sub(1))).rev() {
            let mut should_continue_to_outer = false;
            let complex1 = selectors.get(i).unwrap();
            if is_original(complex1) {
                // Make sure we don't include duplicate originals, which could happen if
                // a style rule extends a component of its own selector.
                for j in 0..num_originals {
                    if result.get(j) == Some(complex1) {
                        rotate_slice(&mut result, 0, j + 1);
                        should_continue_to_outer = true;
                        break;
                    }
                }
                if should_continue_to_outer {
                    continue;
                }
                num_originals += 1;
                result.push_front(complex1.clone());
                continue;
            }

            // The maximum specificity of the sources that caused `complex1` to be
            // generated. In order for `complex1` to be removed, there must be another
            // selector that's a superselector of it *and* that has specificity
            // greater or equal to this.
            let mut max_specificity = 0;
            for component in &complex1.components {
                if let ComplexSelectorComponent::Compound(compound) = component {
                    max_specificity = max_specificity.max(self.source_specificity_for(compound));
                }
            }

            // Look in `result` rather than `selectors` for selectors after `i`. This
            // ensures that we aren't comparing against a selector that's already been
            // trimmed, and thus that if there are two identical selectors only one is
            // trimmed.
            let should_continue = result.iter().any(|complex2| {
                complex2.min_specificity() >= max_specificity
                    && complex2.is_super_selector(complex1)
            });
            if should_continue {
                continue;
            }

            let should_continue = selectors.iter().take(i).any(|complex2| {
                complex2.min_specificity() >= max_specificity
                    && complex2.is_super_selector(complex1)
            });
            if should_continue {
                continue;
            }

            result.push_front(complex1.clone());
        }

        Vec::from(result)
    }

    /// Adds `selector` to this extender.
    ///
    /// Extends `selector` using any registered extensions, then returns the resulting
    /// selector. If any more relevant extensions are added, the returned selector
    /// is automatically updated.
    ///
    /// The `media_query_context` is the media query context in which the selector was
    /// defined, or `None` if it was defined at the top level of the document.
    pub fn add_selector(
        &mut self,
        mut selector: SelectorList,
        // span: Span,
        media_query_context: &Option<Vec<CssMediaQuery>>,
    ) -> ExtendedSelector {
        if !selector.is_invisible() {
            for complex in selector.components.clone() {
                self.originals.insert(&complex);
            }
        }

        if !self.extensions.is_empty() {
            selector = self.extend_list(selector, None, media_query_context);
            /*
              todo: when we have error handling
                  } on SassException catch (error) {
              throw SassException(
                  "From ${error.span.message('')}\n"
                  "${error.message}",
                  span);
            }
              */
        }
        if let Some(mut media_query_context) = media_query_context.clone() {
            self.media_contexts
                .get_mut(&selector)
                .replace(&mut media_query_context);
        }
        let extended_selector = ExtendedSelector::new(selector.clone());
        self.register_selector(selector, &extended_selector);
        extended_selector
    }

    /// Registers the `SimpleSelector`s in `list` to point to `selector` in
    /// `self.selectors`.
    fn register_selector(&mut self, list: SelectorList, selector: &ExtendedSelector) {
        for complex in list.components {
            for component in complex.components {
                if let ComplexSelectorComponent::Compound(component) = component {
                    for simple in component.components {
                        // PERF: we compute the hash twice, which isn't great, but we avoid a superfluous
                        // clone in cases where we have already seen a simple selector (common in
                        // scenarios in which there is a lot of nesting)
                        if let Some(entry) = self.selectors.get_mut(&simple) {
                            entry.insert(selector.clone());
                        } else {
                            self.selectors
                                .entry(simple.clone())
                                .or_insert_with(SelectorHashSet::new)
                                .insert(selector.clone());
                        }

                        if let SimpleSelector::Pseudo(Pseudo {
                            selector: Some(simple_selector),
                            ..
                        }) = simple
                        {
                            self.register_selector(*simple_selector, selector);
                        }
                    }
                }
            }
        }
    }

    /// Adds an extension to this extender.
    ///
    /// The `extender` is the selector for the style rule in which the extension
    /// is defined, and `target` is the selector passed to `@extend`. The `extend`
    /// provides the extend span and indicates whether the extension is optional.
    ///
    /// The `media_context` defines the media query context in which the extension
    /// is defined. It can only extend selectors within the same context. A `None`
    /// context indicates no media queries.
    pub fn add_extension(
        &mut self,
        extender: SelectorList,
        target: &SimpleSelector,
        extend: &ExtendRule,
        media_context: &Option<Vec<CssMediaQuery>>,
        span: Span,
    ) {
        let selectors = self.selectors.get(target).cloned();
        let existing_extensions = self.extensions_by_extender.get(target).cloned();

        let mut new_extensions: Option<IndexMap<ComplexSelector, Extension>> = None;

        for complex in extender.components {
            let state = Extension {
                specificity: complex.max_specificity(),
                extender: complex.clone(),
                target: Some(target.clone()),
                span,
                media_context: media_context.clone(),
                is_optional: extend.is_optional,
                is_original: false,
                left: None,
                right: None,
            };

            let sources = self
                .extensions
                .entry(target.clone())
                .or_insert_with(IndexMap::new);

            if let Some(existing_state) = sources.get(&complex) {
                // If there's already an extend from `extender` to `target`, we don't need
                // to re-run the extension. We may need to mark the extension as
                // mandatory, though.
                let mut new_val = MergedExtension::merge(existing_state.clone(), state).unwrap();
                sources.get_mut(&complex).replace(&mut new_val);
                continue;
            }

            sources.insert(complex.clone(), state.clone());

            for component in complex.components.clone() {
                if let ComplexSelectorComponent::Compound(component) = component {
                    for simple in component.components {
                        self.extensions_by_extender
                            .entry(simple.clone())
                            .or_insert_with(Vec::new)
                            .push(state.clone());
                        // Only source specificity for the original selector is relevant.
                        // Selectors generated by `@extend` don't get new specificity.
                        self.source_specificity
                            .entry(simple.clone())
                            .or_insert_with(|| complex.max_specificity());
                    }
                }
            }

            if selectors.is_some() || existing_extensions.is_some() {
                new_extensions
                    .get_or_insert_with(IndexMap::new)
                    .insert(complex.clone(), state.clone());
            }
        }

        let new_extensions = if let Some(new) = new_extensions {
            new
        } else {
            return;
        };

        let mut new_extensions_by_target = HashMap::new();
        new_extensions_by_target.insert(target.clone(), new_extensions);

        if let Some(existing_extensions) = existing_extensions {
            let additional_extensions =
                self.extend_existing_extensions(existing_extensions, &new_extensions_by_target);
            if let Some(additional_extensions) = additional_extensions {
                map_add_all_2(&mut new_extensions_by_target, additional_extensions);
            }
        }

        if let Some(selectors) = selectors {
            self.extend_existing_selectors(selectors, &new_extensions_by_target);
        }
    }

    /// Extend `extensions` using `new_extensions`.
    ///
    /// Note that this does duplicate some work done by
    /// `Extender::extend_existing_selectors`, but it's necessary to expand each extension's
    /// extender separately without reference to the full selector list, so that
    /// relevant results don't get trimmed too early.
    ///
    /// Returns extensions that should be added to `new_extensions` before
    /// extending selectors in order to properly handle extension loops such as:
    ///```foo
    ///     .c {x: y; @extend .a}
    ///     .x.y.a {@extend .b}
    ///     .z.b {@extend .c}
    ///```
    /// Returns `None` if there are no extensions to add.
    fn extend_existing_extensions(
        &mut self,
        extensions: Vec<Extension>,
        new_extensions: &HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>,
    ) -> Option<HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>> {
        let mut additional_extensions: Option<
            HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>,
        > = None;
        for extension in extensions {
            let mut sources = self
                .extensions
                .get(&extension.target.clone().unwrap())
                .unwrap()
                .clone();

            // `extend_existing_selectors` would have thrown already.
            let selectors: Vec<ComplexSelector> = if let Some(v) = self.extend_complex(
                extension.extender.clone(),
                Some(new_extensions),
                &extension.media_context,
            ) {
                v
            } else {
                continue;
            };
            // todo: when we add error handling, this error is special
            /*
            } on SassException catch (error) {
                throw SassException(
                    "From ${extension.extenderSpan.message('')}\n"
                    "${error.message}",
                    error.span);
            }
            */

            let contains_extension = selectors.first() == Some(&extension.extender);

            let mut first = false;
            for complex in selectors {
                // If the output contains the original complex selector, there's no
                // need to recreate it.
                if contains_extension && first {
                    first = false;
                    continue;
                }

                let with_extender = extension.clone().with_extender(complex.clone());
                let existing_extension = sources.get(&complex);
                if let Some(existing_extension) = existing_extension.cloned() {
                    sources.get_mut(&complex).replace(
                        &mut MergedExtension::merge(existing_extension.clone(), with_extender)
                            .unwrap(),
                    );
                } else {
                    sources
                        .get_mut(&complex)
                        .replace(&mut with_extender.clone());

                    for component in complex.components.clone() {
                        if let ComplexSelectorComponent::Compound(component) = component {
                            for simple in component.components {
                                self.extensions_by_extender
                                    .entry(simple)
                                    .or_insert_with(Vec::new)
                                    .push(with_extender.clone());
                            }
                        }
                    }

                    if new_extensions.contains_key(&extension.target.clone().unwrap()) {
                        additional_extensions
                            .get_or_insert_with(HashMap::new)
                            .entry(extension.target.clone().unwrap())
                            .or_insert_with(IndexMap::new)
                            .insert(complex.clone(), with_extender.clone());
                    }
                }
            }
            // If `selectors` doesn't contain `extension.extender`, for example if it
            // was replaced due to :not() expansion, we must get rid of the old
            // version.
            if !contains_extension {
                // todo: evaluate whether we could get away with swap_remove
                sources.shift_remove(&extension.extender);
            }
        }
        additional_extensions
    }

    /// Extend `extensions` using `new_extensions`.
    fn extend_existing_selectors(
        &mut self,
        selectors: SelectorHashSet,
        new_extensions: &HashMap<SimpleSelector, IndexMap<ComplexSelector, Extension>>,
    ) {
        for mut selector in selectors {
            let old_value = selector.clone().into_selector().0;
            selector.set_inner(self.extend_list(
                old_value.clone(),
                Some(new_extensions),
                &self.media_contexts.get(&old_value).cloned(),
            ));
            /*
            todo: error handling
            } on SassException catch (error) {
            throw SassException(
                "From ${selector.span.message('')}\n"
                "${error.message}",
                error.span);
            }

            */

            // If no extends actually happened (for example because unification
            // failed), we don't need to re-register the selector.
            let selector_as_selector = selector.clone().into_selector().0;
            if old_value == selector_as_selector {
                continue;
            }
            self.register_selector(selector_as_selector, &selector);
        }
    }
}

/// Rotates the element in list from `start` (inclusive) to `end` (exclusive)
/// one index higher, looping the final element back to `start`.
fn rotate_slice<T: Clone>(list: &mut VecDeque<T>, start: usize, end: usize) {
    let mut element = list.get(end - 1).unwrap().clone();
    for i in start..end {
        let next = list.get(i).unwrap().clone();
        list[i] = element;
        element = next;
    }
}

/// Like `HashMap::extend`, but for two-layer maps.
///
/// This avoids copying inner maps from `source` if possible.
fn map_add_all_2<K1: Hash + Eq, K2: Hash + Eq, V>(
    destination: &mut HashMap<K1, IndexMap<K2, V>>,
    source: HashMap<K1, IndexMap<K2, V>>,
) {
    for (key, mut inner) in source {
        if destination.contains_key(&key) {
            destination
                .get_mut(&key)
                .get_or_insert(&mut IndexMap::new())
                .extend(inner);
        } else {
            destination.get_mut(&key).replace(&mut inner);
        }
    }
}
