use std::{
    fmt::{self, Write},
    hash::{Hash, Hasher},
};

use codemap::Span;

use crate::{common::unvendor, error::SassResult};

use super::{
    Attribute, ComplexSelector, ComplexSelectorComponent, CompoundSelector, Namespace,
    QualifiedName, SelectorList, Specificity,
};

const SUBSELECTOR_PSEUDOS: [&str; 6] = [
    "matches",
    "where",
    "is",
    "any",
    "nth-child",
    "nth-last-child",
];

const BASE_SPECIFICITY: i32 = 1000;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum SimpleSelector {
    /// *
    Universal(Namespace),

    /// A pseudo-class or pseudo-element selector.
    ///
    /// The semantics of a specific pseudo selector depends on its name. Some
    /// selectors take arguments, including other selectors. Sass manually encodes
    /// logic for each pseudo selector that takes a selector as an argument, to
    /// ensure that extension and other selector operations work properly.
    Pseudo(Pseudo),

    /// A type selector.
    ///
    /// This selects elements whose name equals the given name.
    Type(QualifiedName),

    /// A placeholder selector.
    ///
    /// This doesn't match any elements. It's intended to be extended using
    /// `@extend`. It's not a plain CSS selector—it should be removed before
    /// emitting a CSS document.
    Placeholder(String),

    /// A selector that matches the parent in the Sass stylesheet.
    /// `&`
    ///
    /// This is not a plain CSS selector—it should be removed before emitting a CSS
    /// document.
    ///
    /// The parameter is the suffix that will be added to the parent selector after
    /// it's been resolved.
    ///
    /// This is assumed to be a valid identifier suffix. It may be `None`,
    /// indicating that the parent selector will not be modified.
    Parent(Option<String>),

    Id(String),

    /// A class selector.
    ///
    /// This selects elements whose `class` attribute contains an identifier with
    /// the given name.
    Class(String),

    Attribute(Box<Attribute>),
}

impl fmt::Display for SimpleSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(name) => write!(f, "#{}", name),
            Self::Class(name) => write!(f, ".{}", name),
            Self::Placeholder(name) => write!(f, "%{}", name),
            Self::Universal(namespace) => write!(f, "{}*", namespace),
            Self::Pseudo(pseudo) => write!(f, "{}", pseudo),
            Self::Type(name) => write!(f, "{}", name),
            Self::Attribute(attr) => write!(f, "{}", attr),
            Self::Parent(..) => unreachable!("It should not be possible to format `&`."),
        }
    }
}

impl SimpleSelector {
    /// The minimum possible specificity that this selector can have.
    ///
    /// Pseudo selectors that contain selectors, like `:not()` and `:matches()`,
    /// can have a range of possible specificities.
    ///
    /// Specifity is represented in base 1000. The spec says this should be
    /// "sufficiently high"; it's extremely unlikely that any single selector
    /// sequence will contain 1000 simple selectors.
    pub fn min_specificity(&self) -> i32 {
        match self {
            Self::Universal(..) => 0,
            Self::Type(..) => 1,
            Self::Pseudo(pseudo) => pseudo.min_specificity(),
            Self::Id(..) => BASE_SPECIFICITY.pow(2_u32),
            _ => BASE_SPECIFICITY,
        }
    }

    /// The maximum possible specificity that this selector can have.
    ///
    /// Pseudo selectors that contain selectors, like `:not()` and `:matches()`,
    /// can have a range of possible specificities.
    pub fn max_specificity(&self) -> i32 {
        match self {
            Self::Universal(..) => 0,
            Self::Pseudo(pseudo) => pseudo.max_specificity(),
            _ => self.min_specificity(),
        }
    }

    pub fn is_invisible(&self) -> bool {
        match self {
            Self::Universal(..)
            | Self::Type(..)
            | Self::Id(..)
            | Self::Class(..)
            | Self::Attribute(..) => false,
            Self::Pseudo(Pseudo { name, selector, .. }) => {
                name != "not" && selector.as_ref().map_or(false, |sel| sel.is_invisible())
            }
            Self::Placeholder(..) => true,
            Self::Parent(..) => unreachable!("parent selectors should be resolved at this point"),
        }
    }

    pub fn add_suffix(&mut self, suffix: &str, span: Span) -> SassResult<()> {
        match self {
            Self::Type(name) => name.ident.push_str(suffix),
            Self::Placeholder(name)
            | Self::Id(name)
            | Self::Class(name)
            | Self::Pseudo(Pseudo {
                name,
                argument: None,
                selector: None,
                ..
            }) => name.push_str(suffix),
            // todo: add test for this?
            _ => return Err((format!("Invalid parent selector \"{}\"", self), span).into()),
        };
        Ok(())
    }

    pub fn is_universal(&self) -> bool {
        matches!(self, Self::Universal(..))
    }

    pub fn is_pseudo(&self) -> bool {
        matches!(self, Self::Pseudo { .. })
    }

    pub fn is_parent(&self) -> bool {
        matches!(self, Self::Parent(..))
    }

    pub fn is_id(&self) -> bool {
        matches!(self, Self::Id(..))
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type(..))
    }

    pub fn unify(self, compound: Vec<Self>) -> Option<Vec<Self>> {
        match self {
            Self::Type(..) => self.unify_type(compound),
            Self::Universal(..) => self.unify_universal(compound),
            Self::Pseudo { .. } => self.unify_pseudo(compound),
            Self::Id(..) => {
                if compound
                    .iter()
                    .any(|simple| simple.is_id() && simple != &self)
                {
                    return None;
                }

                self.unify_default(compound)
            }
            _ => self.unify_default(compound),
        }
    }

    /// Returns the components of a `CompoundSelector` that matches only elements
    /// matched by both this and `compound`.
    ///
    /// By default, this just returns a copy of `compound` with this selector
    /// added to the end, or returns the original array if this selector already
    /// exists in it.
    ///
    /// Returns `None` if unification is impossible—for example, if there are
    /// multiple ID selectors.
    fn unify_default(self, mut compound: Vec<Self>) -> Option<Vec<Self>> {
        if compound.len() == 1 && compound[0].is_universal() {
            return compound.swap_remove(0).unify(vec![self]);
        }
        if compound.contains(&self) {
            return Some(compound);
        }
        let mut result: Vec<SimpleSelector> = Vec::new();
        let mut added_this = false;
        for simple in compound {
            if !added_this && simple.is_pseudo() {
                result.push(self.clone());
                added_this = true;
            }
            result.push(simple);
        }

        if !added_this {
            result.push(self);
        }

        Some(result)
    }

    fn unify_universal(self, mut compound: Vec<Self>) -> Option<Vec<Self>> {
        if let Self::Universal(..) | Self::Type(..) = compound[0] {
            let mut unified = vec![self.unify_universal_and_element(&compound[0])?];
            unified.extend(compound.into_iter().skip(1));
            return Some(unified);
        }

        if self != Self::Universal(Namespace::Asterisk) && self != Self::Universal(Namespace::None)
        {
            let mut v = vec![self];
            v.append(&mut compound);
            return Some(v);
        }

        if !compound.is_empty() {
            return Some(compound);
        }

        Some(vec![self])
    }

    /// Returns a `SimpleSelector` that matches only elements that are matched by
    /// both `selector1` and `selector2`, which must both be either
    /// `SimpleSelector::Universal`s or `SimpleSelector::Type`s.
    ///
    /// If no such selector can be produced, returns `None`.
    fn unify_universal_and_element(&self, other: &Self) -> Option<Self> {
        let namespace1;
        let name1;
        if let SimpleSelector::Type(name) = self.clone() {
            namespace1 = name.namespace;
            name1 = name.ident;
        } else if let SimpleSelector::Universal(namespace) = self.clone() {
            namespace1 = namespace;
            name1 = String::new();
        } else {
            unreachable!("{:?} must be a universal selector or a type selector", self);
        }

        let namespace2;
        let mut name2 = String::new();

        if let SimpleSelector::Universal(namespace) = other {
            namespace2 = namespace.clone();
        } else if let SimpleSelector::Type(name) = other {
            namespace2 = name.namespace.clone();
            name2 = name.ident.clone();
        } else {
            unreachable!(
                "{:?} must be a universal selector or a type selector",
                other
            );
        }

        let namespace = if namespace1 == namespace2 || namespace2 == Namespace::Asterisk {
            namespace1
        } else if namespace1 == Namespace::Asterisk {
            namespace2
        } else {
            return None;
        };

        let name = if name1 == name2 || name2.is_empty() {
            name1
        } else if name1.is_empty() || name1 == "*" {
            name2
        } else {
            return None;
        };

        Some(if name.is_empty() {
            SimpleSelector::Universal(namespace)
        } else {
            SimpleSelector::Type(QualifiedName {
                namespace,
                ident: name,
            })
        })
    }

    fn unify_type(self, mut compound: Vec<Self>) -> Option<Vec<Self>> {
        if let Self::Universal(..) | Self::Type(..) = compound[0] {
            let mut unified = vec![self.unify_universal_and_element(&compound[0])?];
            unified.extend(compound.into_iter().skip(1));
            Some(unified)
        } else {
            let mut unified = vec![self];
            unified.append(&mut compound);
            Some(unified)
        }
    }

    fn unify_pseudo(self, mut compound: Vec<Self>) -> Option<Vec<Self>> {
        if compound.len() == 1 && compound[0].is_universal() {
            return compound.remove(0).unify(vec![self]);
        }
        if compound.contains(&self) {
            return Some(compound);
        }

        let mut result = Vec::new();

        let mut added_self = false;

        for simple in compound {
            if let Self::Pseudo(Pseudo {
                is_class: false, ..
            }) = simple
            {
                // A given compound selector may only contain one pseudo element. If
                // `compound` has a different one than `self`, unification fails.
                if let Self::Pseudo(Pseudo {
                    is_class: false, ..
                }) = self
                {
                    return None;
                }

                // Otherwise, this is a pseudo selector and should come before pseduo
                // elements.
                result.push(self.clone());
                added_self = true;
            }
            result.push(simple);
        }

        if !added_self {
            result.push(self);
        }

        Some(result)
    }

    pub fn is_super_selector_of_compound(&self, compound: &CompoundSelector) -> bool {
        compound.components.iter().any(|their_simple| {
            if self == their_simple {
                return true;
            }
            if let SimpleSelector::Pseudo(Pseudo {
                selector: Some(sel),
                name,
                ..
            }) = their_simple
            {
                if SUBSELECTOR_PSEUDOS.contains(&unvendor(name)) {
                    return sel.components.iter().all(|complex| {
                        if complex.components.len() != 1 {
                            return false;
                        };
                        complex
                            .components
                            .first()
                            .unwrap()
                            .as_compound()
                            .components
                            .contains(self)
                    });
                }
                false
            } else {
                false
            }
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Pseudo {
    /// The name of this selector.
    pub name: String,

    /// Whether this is a pseudo-class selector.
    ///
    /// If this is false, this is a pseudo-element selector
    pub is_class: bool,

    /// Whether this is syntactically a pseudo-class selector.
    ///
    /// This is the same as `is_class` unless this selector is a pseudo-element
    /// that was written syntactically as a pseudo-class (`:before`, `:after`,
    /// `:first-line`, or `:first-letter`).
    ///
    /// If this is false, it is syntactically a psuedo-element
    pub is_syntactic_class: bool,

    /// The non-selector argument passed to this selector.
    ///
    /// This is `None` if there's no argument. If `argument` and `selector` are
    /// both non-`None`, the selector follows the argument.
    pub argument: Option<Box<str>>,

    /// The selector argument passed to this selector.
    ///
    /// This is `None` if there's no selector. If `argument` and `selector` are
    /// both non-`None`, the selector follows the argument.
    pub selector: Option<Box<SelectorList>>,

    pub span: Span,
}

impl PartialEq for Pseudo {
    fn eq(&self, other: &Pseudo) -> bool {
        self.name == other.name
            && self.is_class == other.is_class
            && self.argument == other.argument
            && self.selector == other.selector
    }
}

impl Eq for Pseudo {}

impl Hash for Pseudo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.is_class.hash(state);
        self.argument.hash(state);
        self.selector.hash(state);
    }
}

impl fmt::Display for Pseudo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sel) = &self.selector {
            if self.name == "not" && sel.is_invisible() {
                return Ok(());
            }
        }

        f.write_char(':')?;

        if !self.is_syntactic_class {
            f.write_char(':')?;
        }

        f.write_str(&self.name)?;

        if self.argument.is_none() && self.selector.is_none() {
            return Ok(());
        }

        f.write_char('(')?;
        if let Some(arg) = &self.argument {
            f.write_str(arg)?;
            if self.selector.is_some() {
                f.write_char(' ')?;
            }
        }

        if let Some(sel) = &self.selector {
            write!(f, "{}", sel)?;
        }

        f.write_char(')')
    }
}

impl Pseudo {
    /// Returns whether `pseudo1` is a superselector of `compound2`.
    ///
    /// That is, whether `pseudo1` matches every element that `compound2` matches, as well
    /// as possibly additional elements.
    ///
    /// This assumes that `pseudo1`'s `selector` argument is not `None`.
    ///
    /// If `parents` is passed, it represents the parents of `compound`. This is
    /// relevant for pseudo selectors with selector arguments, where we may need to
    /// know if the parent selectors in the selector argument match `parents`.
    pub fn is_super_selector(
        &self,
        compound: &CompoundSelector,
        parents: Option<Vec<ComplexSelectorComponent>>,
    ) -> bool {
        debug_assert!(self.selector.is_some());
        match self.normalized_name() {
            "matches" | "is" | "any" | "where" => {
                selector_pseudos_named(compound.clone(), &self.name, true).any(move |pseudo2| {
                    self.selector
                        .as_ref()
                        .unwrap()
                        .is_superselector(&pseudo2.selector.unwrap())
                }) || self
                    .selector
                    .as_ref()
                    .unwrap()
                    .components
                    .iter()
                    .any(move |complex1| {
                        let mut components = parents.clone().unwrap_or_default();
                        components.push(ComplexSelectorComponent::Compound(compound.clone()));
                        complex1.is_super_selector(&ComplexSelector::new(components, false))
                    })
            }
            "has" | "host" | "host-context" => {
                selector_pseudos_named(compound.clone(), &self.name, true).any(|pseudo2| {
                    self.selector
                        .as_ref()
                        .unwrap()
                        .is_superselector(&pseudo2.selector.unwrap())
                })
            }
            "slotted" => {
                selector_pseudos_named(compound.clone(), &self.name, false).any(|pseudo2| {
                    self.selector
                        .as_ref()
                        .unwrap()
                        .is_superselector(pseudo2.selector.as_ref().unwrap())
                })
            }
            "not" => self
                .selector
                .as_ref()
                .unwrap()
                .components
                .iter()
                .all(|complex| {
                    compound.components.iter().any(|simple2| {
                        if let SimpleSelector::Type(..) = simple2 {
                            let compound1 = complex.components.last();
                            if let Some(ComplexSelectorComponent::Compound(c)) = compound1 {
                                c.components
                                    .iter()
                                    .any(|simple1| simple1.is_type() && simple1 != simple2)
                            } else {
                                false
                            }
                        } else if let SimpleSelector::Id(..) = simple2 {
                            let compound1 = complex.components.last();
                            if let Some(ComplexSelectorComponent::Compound(c)) = compound1 {
                                c.components
                                    .iter()
                                    .any(|simple1| simple1.is_id() && simple1 != simple2)
                            } else {
                                false
                            }
                        } else if let SimpleSelector::Pseudo(Pseudo {
                            selector: Some(sel),
                            name,
                            ..
                        }) = simple2
                        {
                            if name != &self.name {
                                return false;
                            }
                            sel.is_superselector(&SelectorList {
                                components: vec![complex.clone()],
                                span: self.span,
                            })
                        } else {
                            false
                        }
                    })
                }),
            "current" => selector_pseudos_named(compound.clone(), &self.name, self.is_class)
                .any(|pseudo2| self.selector == pseudo2.selector),
            "nth-child" | "nth-last-child" => compound.components.iter().any(|pseudo2| {
                if let SimpleSelector::Pseudo(
                    pseudo @ Pseudo {
                        selector: Some(..), ..
                    },
                ) = pseudo2
                {
                    pseudo.name == self.name
                        && pseudo.argument == self.argument
                        && self
                            .selector
                            .as_ref()
                            .unwrap()
                            .is_superselector(pseudo.selector.as_ref().unwrap())
                } else {
                    false
                }
            }),
            _ => unreachable!(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_selector(self, selector: Option<Box<SelectorList>>) -> Self {
        Self { selector, ..self }
    }

    pub fn max_specificity(&self) -> i32 {
        self.specificity().max
    }

    pub fn min_specificity(&self) -> i32 {
        self.specificity().min
    }

    pub fn specificity(&self) -> Specificity {
        if !self.is_class {
            return Specificity { min: 1, max: 1 };
        }

        let selector = match &self.selector {
            Some(sel) => sel,
            None => {
                return Specificity {
                    min: BASE_SPECIFICITY,
                    max: BASE_SPECIFICITY,
                }
            }
        };

        if self.name == "not" {
            let mut min = 0;
            let mut max = 0;
            for complex in &selector.components {
                min = min.max(complex.min_specificity());
                max = max.max(complex.max_specificity());
            }
            Specificity { min, max }
        } else {
            // This is higher than any selector's specificity can actually be.
            let mut min = BASE_SPECIFICITY.pow(3_u32);
            let mut max = 0;
            for complex in &selector.components {
                min = min.min(complex.min_specificity());
                max = max.max(complex.max_specificity());
            }
            Specificity { min, max }
        }
    }

    /// Like `name`, but without any vendor prefixes.
    pub fn normalized_name(&self) -> &str {
        unvendor(&self.name)
    }
}

/// Returns all pseudo selectors in `compound` that have a selector argument,
/// and that have the given `name`.
fn selector_pseudos_named(
    compound: CompoundSelector,
    name: &str,
    is_class: bool,
) -> impl Iterator<Item = Pseudo> + '_ {
    compound
        .components
        .into_iter()
        .filter_map(|c| {
            if let SimpleSelector::Pseudo(p) = c {
                Some(p)
            } else {
                None
            }
        })
        .filter(move |p| p.is_class == is_class && p.selector.is_some() && p.name == name)
}
