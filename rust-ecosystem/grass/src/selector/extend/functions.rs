#![allow(clippy::similar_names)]

use std::collections::VecDeque;

use super::super::{
    Combinator, ComplexSelector, ComplexSelectorComponent, CompoundSelector, Pseudo, SimpleSelector,
};

/// Returns the contents of a `SelectorList` that matches only elements that are
/// matched by both `complex_one` and `complex_two`.
///
/// If no such list can be produced, returns `None`.
pub(crate) fn unify_complex(
    complexes: Vec<Vec<ComplexSelectorComponent>>,
) -> Option<Vec<Vec<ComplexSelectorComponent>>> {
    debug_assert!(!complexes.is_empty());

    if complexes.len() == 1 {
        return Some(complexes);
    }

    let mut unified_base: Option<Vec<SimpleSelector>> = None;

    for complex in &complexes {
        let base = complex.last()?;

        if let ComplexSelectorComponent::Compound(base) = base {
            if let Some(mut some_unified_base) = unified_base.clone() {
                for simple in base.components.clone() {
                    some_unified_base = simple.unify(some_unified_base.clone())?;
                }
                unified_base = Some(some_unified_base);
            } else {
                unified_base = Some(base.components.clone());
            }
        } else {
            return None;
        }
    }

    let mut complexes_without_bases: Vec<Vec<ComplexSelectorComponent>> = complexes
        .into_iter()
        .map(|mut complex| {
            complex.pop();
            complex
        })
        .collect();

    complexes_without_bases
        .last_mut()
        .unwrap()
        .push(ComplexSelectorComponent::Compound(CompoundSelector {
            components: unified_base?,
        }));

    Some(weave(complexes_without_bases))
}

/// Expands "parenthesized selectors" in `complexes`.
///
/// That is, if we have `.A .B {@extend .C}` and `.D .C {...}`, this
/// conceptually expands into `.D .C, .D (.A .B)`, and this function translates
/// `.D (.A .B)` into `.D .A .B, .A .D .B`. For thoroughness, `.A.D .B` would
/// also be required, but including merged selectors results in exponential
/// output for very little gain.
///
/// The selector `.D (.A .B)` is represented as the list `[[.D], [.A, .B]]`.
pub(crate) fn weave(
    mut complexes: Vec<Vec<ComplexSelectorComponent>>,
) -> Vec<Vec<ComplexSelectorComponent>> {
    let mut prefixes: Vec<Vec<ComplexSelectorComponent>> = vec![complexes.remove(0)];

    for mut complex in complexes {
        let target = match complex.pop() {
            Some(c) => c,
            None => continue,
        };

        if complex.is_empty() {
            for prefix in &mut prefixes {
                prefix.push(target.clone());
            }
            continue;
        }

        let parents: Vec<ComplexSelectorComponent> = complex;
        let mut new_prefixes: Vec<Vec<ComplexSelectorComponent>> = Vec::new();

        for prefix in prefixes {
            if let Some(parent_prefixes) = weave_parents(prefix, parents.clone()) {
                for mut parent_prefix in parent_prefixes {
                    parent_prefix.push(target.clone());
                    new_prefixes.push(parent_prefix);
                }
            }
        }
        prefixes = new_prefixes;
    }

    prefixes
}

/// Interweaves `parents_one` and `parents_two` as parents of the same target selector.
///
/// Returns all possible orderings of the selectors in the inputs (including
/// using unification) that maintain the relative ordering of the input. For
/// example, given `.foo .bar` and `.baz .bang`, this would return `.foo .bar
/// .baz .bang`, `.foo .bar.baz .bang`, `.foo .baz .bar .bang`, `.foo .baz
/// .bar.bang`, `.foo .baz .bang .bar`, and so on until `.baz .bang .foo .bar`.
///
/// Semantically, for selectors A and B, this returns all selectors `AB_i`
/// such that the union over all i of elements matched by `AB_i X` is
/// identical to the intersection of all elements matched by `A X` and all
/// elements matched by `B X`. Some `AB_i` are elided to reduce the size of
/// the output.
fn weave_parents(
    parents_one: Vec<ComplexSelectorComponent>,
    parents_two: Vec<ComplexSelectorComponent>,
) -> Option<Vec<Vec<ComplexSelectorComponent>>> {
    let mut queue_one = VecDeque::from(parents_one);
    let mut queue_two = VecDeque::from(parents_two);

    let initial_combinators = merge_initial_combinators(&mut queue_one, &mut queue_two)?;

    let mut final_combinators = merge_final_combinators(&mut queue_one, &mut queue_two, None)?;

    match (first_if_root(&mut queue_one), first_if_root(&mut queue_two)) {
        (Some(root_one), Some(root_two)) => {
            let root = ComplexSelectorComponent::Compound(root_one.unify(root_two)?);
            queue_one.push_front(root.clone());
            queue_two.push_front(root);
        }
        (Some(root_one), None) => {
            queue_two.push_front(ComplexSelectorComponent::Compound(root_one));
        }
        (None, Some(root_two)) => {
            queue_one.push_front(ComplexSelectorComponent::Compound(root_two));
        }
        (None, None) => {}
    }

    let mut groups_one = group_selectors(Vec::from(queue_one));
    let mut groups_two = group_selectors(Vec::from(queue_two));

    let lcs = longest_common_subsequence(
        groups_two.as_slices().0,
        groups_one.as_slices().0,
        Some(&|group_one, group_two| {
            if group_one == group_two {
                return Some(group_one);
            }

            if let ComplexSelectorComponent::Combinator(..) = group_one.first()? {
                return None;
            }
            if let ComplexSelectorComponent::Combinator(..) = group_two.first()? {
                return None;
            }

            if complex_is_parent_superselector(group_one.clone(), group_two.clone()) {
                return Some(group_two);
            }
            if complex_is_parent_superselector(group_two.clone(), group_one.clone()) {
                return Some(group_one);
            }

            if !must_unify(&group_one, &group_two) {
                return None;
            }

            let unified = unify_complex(vec![group_one, group_two])?;
            if unified.len() > 1 {
                return None;
            }

            unified.first().cloned()
        }),
    );

    let mut choices = vec![vec![initial_combinators
        .into_iter()
        .map(ComplexSelectorComponent::Combinator)
        .collect::<Vec<ComplexSelectorComponent>>()]];

    for group in lcs {
        choices.push(
            chunks(&mut groups_one, &mut groups_two, |sequence| {
                complex_is_parent_superselector(
                    match sequence.front() {
                        Some(v) => v.clone(),
                        None => return true,
                    },
                    group.clone(),
                )
            })
            .into_iter()
            .map(|chunk| chunk.into_iter().flatten().collect())
            .collect(),
        );
        choices.push(vec![group]);
        groups_one.pop_front();
        groups_two.pop_front();
    }

    choices.push(
        chunks(&mut groups_one, &mut groups_two, VecDeque::is_empty)
            .into_iter()
            .map(|chunk| chunk.into_iter().flatten().collect())
            .collect(),
    );

    choices.append(&mut final_combinators);

    Some(
        paths(
            choices
                .into_iter()
                .filter(|choice| !choice.is_empty())
                .collect(),
        )
        .into_iter()
        .map(|chunk| chunk.into_iter().flatten().collect())
        .collect(),
    )
}

/// Extracts leading `Combinator`s from `components_one` and `components_two` and
/// merges them together into a single list of combinators.
///
/// If there are no combinators to be merged, returns an empty list. If the
/// combinators can't be merged, returns `None`.
fn merge_initial_combinators(
    components_one: &mut VecDeque<ComplexSelectorComponent>,
    components_two: &mut VecDeque<ComplexSelectorComponent>,
) -> Option<Vec<Combinator>> {
    let mut combinators_one: Vec<Combinator> = Vec::new();

    while let Some(ComplexSelectorComponent::Combinator(c)) = components_one.front() {
        combinators_one.push(*c);
        components_one.pop_front();
    }

    let mut combinators_two = Vec::new();

    while let Some(ComplexSelectorComponent::Combinator(c)) = components_two.front() {
        combinators_two.push(*c);
        components_two.pop_front();
    }

    let lcs = longest_common_subsequence(&combinators_one, &combinators_two, None);

    if lcs == combinators_one {
        Some(combinators_two)
    } else if lcs == combinators_two {
        Some(combinators_one)
    } else {
        // If neither sequence of combinators is a subsequence of the other, they
        // cannot be merged successfully.
        None
    }
}

/// Returns the longest common subsequence between `list_one` and `list_two`.
///
/// If there are more than one equally long common subsequence, returns the one
/// which starts first in `list_one`.
///
/// If `select` is passed, it's used to check equality between elements in each
/// list. If it returns `None`, the elements are considered unequal; otherwise,
/// it should return the element to include in the return value.
fn longest_common_subsequence<T: PartialEq + Clone>(
    list_one: &[T],
    list_two: &[T],
    select: Option<&dyn Fn(T, T) -> Option<T>>,
) -> Vec<T> {
    let select = select.unwrap_or(&|element_one, element_two| {
        if element_one == element_two {
            Some(element_one)
        } else {
            None
        }
    });

    let mut lengths = vec![vec![0; list_two.len() + 1]; list_one.len() + 1];

    let mut selections: Vec<Vec<Option<T>>> = vec![vec![None; list_two.len()]; list_one.len()];

    for i in 0..list_one.len() {
        for j in 0..list_two.len() {
            let selection = select(
                list_one.get(i).unwrap().clone(),
                list_two.get(j).unwrap().clone(),
            );
            selections[i][j] = selection.clone();
            lengths[i + 1][j + 1] = if selection.is_none() {
                std::cmp::max(lengths[i + 1][j], lengths[i][j + 1])
            } else {
                lengths[i][j] + 1
            };
        }
    }

    fn backtrack<T: Clone>(
        i: isize,
        j: isize,
        lengths: Vec<Vec<i32>>,
        selections: &mut Vec<Vec<Option<T>>>,
    ) -> Vec<T> {
        if i == -1 || j == -1 {
            return Vec::new();
        }

        let selection = selections.get(i as usize).cloned().unwrap_or_default();

        if let Some(Some(selection)) = selection.get(j as usize) {
            let mut tmp = backtrack(i - 1, j - 1, lengths, selections);
            tmp.push(selection.clone());
            return tmp;
        }

        if lengths[(i + 1) as usize][j as usize] > lengths[i as usize][(j + 1) as usize] {
            backtrack(i, j - 1, lengths, selections)
        } else {
            backtrack(i - 1, j, lengths, selections)
        }
    }
    backtrack(
        (list_one.len() as isize).saturating_sub(1),
        (list_two.len() as isize).saturating_sub(1),
        lengths,
        &mut selections,
    )
}

/// Extracts trailing `Combinator`s, and the selectors to which they apply, from
/// `components_one` and `components_two` and merges them together into a single list.
///
/// If there are no combinators to be merged, returns an empty list. If the
/// sequences can't be merged, returns `None`.
#[allow(clippy::cognitive_complexity)]
fn merge_final_combinators(
    components_one: &mut VecDeque<ComplexSelectorComponent>,
    components_two: &mut VecDeque<ComplexSelectorComponent>,
    result: Option<VecDeque<Vec<Vec<ComplexSelectorComponent>>>>,
) -> Option<Vec<Vec<Vec<ComplexSelectorComponent>>>> {
    let mut result = result.unwrap_or_default();

    if (components_one.is_empty() || !components_one.back().unwrap().is_combinator())
        && (components_two.is_empty() || !components_two.back().unwrap().is_combinator())
    {
        return Some(Vec::from(result));
    }

    let mut combinators_one = Vec::new();

    while let Some(ComplexSelectorComponent::Combinator(combinator)) =
        components_one.get(components_one.len().saturating_sub(1))
    {
        combinators_one.push(*combinator);
        components_one.pop_back();
    }

    let mut combinators_two = Vec::new();

    while let Some(ComplexSelectorComponent::Combinator(combinator)) =
        components_two.get(components_two.len().saturating_sub(1))
    {
        combinators_two.push(*combinator);
        components_two.pop_back();
    }

    if combinators_one.len() > 1 || combinators_two.len() > 1 {
        // If there are multiple combinators, something hacky's going on. If one
        // is a supersequence of the other, use that, otherwise give up.
        let lcs = longest_common_subsequence(&combinators_one, &combinators_two, None);
        if lcs == combinators_one {
            result.push_front(vec![combinators_two
                .into_iter()
                .map(ComplexSelectorComponent::Combinator)
                .rev()
                .collect()]);
        } else if lcs == combinators_two {
            result.push_front(vec![combinators_one
                .into_iter()
                .map(ComplexSelectorComponent::Combinator)
                .rev()
                .collect()]);
        } else {
            return None;
        }

        return Some(Vec::from(result));
    }

    let combinator_one = combinators_one.first();

    let combinator_two = combinators_two.first();

    // This code looks complicated, but it's actually just a bunch of special
    // cases for interactions between different combinators.
    match (combinator_one, combinator_two) {
        (Some(combinator_one), Some(combinator_two)) => {
            let compound_one = match components_one.pop_back() {
                Some(ComplexSelectorComponent::Compound(c)) => c,
                Some(..) | None => unreachable!(),
            };
            let compound_two = match components_two.pop_back() {
                Some(ComplexSelectorComponent::Compound(c)) => c,
                Some(..) | None => unreachable!(),
            };

            match (combinator_one, combinator_two) {
                (Combinator::FollowingSibling, Combinator::FollowingSibling) => {
                    if compound_one.is_super_selector(&compound_two, &None) {
                        result.push_front(vec![vec![
                            ComplexSelectorComponent::Compound(compound_two),
                            ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                        ]]);
                    } else if compound_two.is_super_selector(&compound_one, &None) {
                        result.push_front(vec![vec![
                            ComplexSelectorComponent::Compound(compound_one),
                            ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                        ]]);
                    } else {
                        let mut choices = vec![
                            vec![
                                ComplexSelectorComponent::Compound(compound_one.clone()),
                                ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                                ComplexSelectorComponent::Compound(compound_two.clone()),
                                ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                            ],
                            vec![
                                ComplexSelectorComponent::Compound(compound_two.clone()),
                                ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                                ComplexSelectorComponent::Compound(compound_one.clone()),
                                ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                            ],
                        ];

                        if let Some(unified) = compound_one.unify(compound_two) {
                            choices.push(vec![
                                ComplexSelectorComponent::Compound(unified),
                                ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                            ]);
                        }

                        result.push_front(choices);
                    }
                }
                (Combinator::FollowingSibling, Combinator::NextSibling)
                | (Combinator::NextSibling, Combinator::FollowingSibling) => {
                    let following_sibling_selector =
                        if combinator_one == &Combinator::FollowingSibling {
                            compound_one.clone()
                        } else {
                            compound_two.clone()
                        };

                    let next_sibling_selector = if combinator_one == &Combinator::FollowingSibling {
                        compound_two.clone()
                    } else {
                        compound_one.clone()
                    };

                    if following_sibling_selector.is_super_selector(&next_sibling_selector, &None) {
                        result.push_front(vec![vec![
                            ComplexSelectorComponent::Compound(next_sibling_selector),
                            ComplexSelectorComponent::Combinator(Combinator::NextSibling),
                        ]]);
                    } else {
                        let mut v = vec![vec![
                            ComplexSelectorComponent::Compound(following_sibling_selector),
                            ComplexSelectorComponent::Combinator(Combinator::FollowingSibling),
                            ComplexSelectorComponent::Compound(next_sibling_selector),
                            ComplexSelectorComponent::Combinator(Combinator::NextSibling),
                        ]];

                        if let Some(unified) = compound_one.unify(compound_two) {
                            v.push(vec![
                                ComplexSelectorComponent::Compound(unified),
                                ComplexSelectorComponent::Combinator(Combinator::NextSibling),
                            ]);
                        }
                        result.push_front(v);
                    }
                }
                (Combinator::Child, Combinator::NextSibling)
                | (Combinator::Child, Combinator::FollowingSibling) => {
                    result.push_front(vec![vec![
                        ComplexSelectorComponent::Compound(compound_two),
                        ComplexSelectorComponent::Combinator(*combinator_two),
                    ]]);
                    components_one.push_back(ComplexSelectorComponent::Compound(compound_one));
                    components_one
                        .push_back(ComplexSelectorComponent::Combinator(Combinator::Child));
                }
                (Combinator::NextSibling, Combinator::Child)
                | (Combinator::FollowingSibling, Combinator::Child) => {
                    result.push_front(vec![vec![
                        ComplexSelectorComponent::Compound(compound_one),
                        ComplexSelectorComponent::Combinator(*combinator_one),
                    ]]);
                    components_two.push_back(ComplexSelectorComponent::Compound(compound_two));
                    components_two
                        .push_back(ComplexSelectorComponent::Combinator(Combinator::Child));
                }
                (..) => {
                    if combinator_one != combinator_two {
                        return None;
                    }

                    let unified = compound_one.unify(compound_two)?;

                    result.push_front(vec![vec![
                        ComplexSelectorComponent::Compound(unified),
                        ComplexSelectorComponent::Combinator(*combinator_one),
                    ]]);
                }
            }

            merge_final_combinators(components_one, components_two, Some(result))
        }
        (Some(combinator_one), None) => {
            if *combinator_one == Combinator::Child && !components_two.is_empty() {
                if let Some(ComplexSelectorComponent::Compound(c1)) = components_one.back() {
                    if let Some(ComplexSelectorComponent::Compound(c2)) = components_two.back() {
                        if c2.is_super_selector(c1, &None) {
                            components_two.pop_back();
                        }
                    }
                }
            }

            result.push_front(vec![vec![
                components_one.pop_back().unwrap(),
                ComplexSelectorComponent::Combinator(*combinator_one),
            ]]);

            merge_final_combinators(components_one, components_two, Some(result))
        }
        (None, Some(combinator_two)) => {
            if *combinator_two == Combinator::Child && !components_one.is_empty() {
                if let Some(ComplexSelectorComponent::Compound(c1)) = components_one.back() {
                    if let Some(ComplexSelectorComponent::Compound(c2)) = components_two.back() {
                        if c1.is_super_selector(c2, &None) {
                            components_one.pop_back();
                        }
                    }
                }
            }

            result.push_front(vec![vec![
                components_two.pop_back().unwrap(),
                ComplexSelectorComponent::Combinator(*combinator_two),
            ]]);
            merge_final_combinators(components_one, components_two, Some(result))
        }
        (None, None) => unreachable!(),
    }
}

/// If the first element of `queue` has a `::root` selector, removes and returns
/// that element.
fn first_if_root(queue: &mut VecDeque<ComplexSelectorComponent>) -> Option<CompoundSelector> {
    if queue.is_empty() {
        return None;
    }
    if let Some(ComplexSelectorComponent::Compound(c)) = queue.front() {
        if !has_root(c) {
            return None;
        }
        let compound = c.clone();
        queue.pop_front();
        Some(compound)
    } else {
        None
    }
}

/// Returns whether or not `compound` contains a `::root` selector.
fn has_root(compound: &CompoundSelector) -> bool {
    compound.components.iter().any(|simple| {
        if let SimpleSelector::Pseudo(pseudo) = simple {
            pseudo.is_class && pseudo.normalized_name() == "root"
        } else {
            false
        }
    })
}

/// Returns `complex`, grouped into sub-lists such that no sub-list contains two
/// adjacent `ComplexSelector`s.
///
/// For example, `(A B > C D + E ~ > G)` is grouped into
/// `[(A) (B > C) (D + E ~ > G)]`.
fn group_selectors(
    complex: Vec<ComplexSelectorComponent>,
) -> VecDeque<Vec<ComplexSelectorComponent>> {
    let mut groups = VecDeque::new();

    let mut iter = complex.into_iter();

    groups.push_back(if let Some(c) = iter.next() {
        vec![c]
    } else {
        return groups;
    });

    for c in iter {
        let mut last_group = groups.pop_back().unwrap();
        if last_group
            .last()
            .map_or(false, ComplexSelectorComponent::is_combinator)
            || c.is_combinator()
        {
            last_group.push(c);
            groups.push_back(last_group);
        } else {
            groups.push_back(last_group);
            groups.push_back(vec![c]);
        }
    }

    groups
}

/// Returns all orderings of initial subseqeuences of `queue_one` and `queue_two`.
///
/// The `done` callback is used to determine the extent of the initial
/// subsequences. It's called with each queue until it returns `true`.
///
/// This destructively removes the initial subsequences of `queue_one` and
/// `queue_two`.
///
/// For example, given `(A B C | D E)` and `(1 2 | 3 4 5)` (with `|` denoting
/// the boundary of the initial subsequence), this would return `[(A B C 1 2),
/// (1 2 A B C)]`. The queues would then contain `(D E)` and `(3 4 5)`.
fn chunks<T: Clone>(
    queue_one: &mut VecDeque<T>,
    queue_two: &mut VecDeque<T>,
    done: impl Fn(&VecDeque<T>) -> bool,
) -> Vec<Vec<T>> {
    let mut chunk_one = Vec::new();
    while !done(queue_one) {
        chunk_one.push(queue_one.pop_front().unwrap());
    }

    let mut chunk_two = Vec::new();
    while !done(queue_two) {
        chunk_two.push(queue_two.pop_front().unwrap());
    }

    match (chunk_one.is_empty(), chunk_two.is_empty()) {
        (true, true) => Vec::new(),
        (true, false) => vec![chunk_two],
        (false, true) => vec![chunk_one],
        (false, false) => {
            let mut l1 = chunk_one.clone();
            l1.append(&mut chunk_two.clone());

            let mut l2 = chunk_two;
            l2.append(&mut chunk_one);

            vec![l1, l2]
        }
    }
}

/// Like `complex_is_superselector`, but compares `complex_one` and `complex_two` as
/// though they shared an implicit base `SimpleSelector`.
///
/// For example, `B` is not normally a superselector of `B A`, since it doesn't
/// match elements that match `A`. However, it *is* a parent superselector,
/// since `B X` is a superselector of `B A X`.
fn complex_is_parent_superselector(
    mut complex_one: Vec<ComplexSelectorComponent>,
    mut complex_two: Vec<ComplexSelectorComponent>,
) -> bool {
    if let Some(ComplexSelectorComponent::Combinator(..)) = complex_one.first() {
        return false;
    }
    if let Some(ComplexSelectorComponent::Combinator(..)) = complex_two.first() {
        return false;
    }
    if complex_one.len() > complex_two.len() {
        return false;
    }
    let base = CompoundSelector {
        components: vec![SimpleSelector::Placeholder(String::new())],
    };
    complex_one.push(ComplexSelectorComponent::Compound(base.clone()));
    complex_two.push(ComplexSelectorComponent::Compound(base));

    ComplexSelector::new(complex_one, false)
        .is_super_selector(&ComplexSelector::new(complex_two, false))
}

/// Returns a list of all possible paths through the given lists.
///
/// For example, given `[[1, 2], [3, 4], [5]]`, this returns:
///
/// ```no_run
/// [[1, 3, 5],
///  [2, 3, 5],
///  [1, 4, 5],
///  [2, 4, 5]];
/// ```
pub(crate) fn paths<T: Clone>(choices: Vec<Vec<T>>) -> Vec<Vec<T>> {
    choices.into_iter().fold(vec![vec![]], |paths, choice| {
        choice
            .into_iter()
            .flat_map(move |option| {
                paths.clone().into_iter().map(move |mut path| {
                    path.push(option.clone());
                    path
                })
            })
            .collect()
    })
}

/// Returns whether `complex_one` and `complex_two` need to be unified to produce a
/// valid combined selector.
///
/// This is necessary when both selectors contain the same unique simple
/// selector, such as an ID.
fn must_unify(
    complex_one: &[ComplexSelectorComponent],
    complex_two: &[ComplexSelectorComponent],
) -> bool {
    let mut unique_selectors = Vec::new();
    for component in complex_one {
        if let ComplexSelectorComponent::Compound(c) = component {
            unique_selectors.extend(c.components.iter().filter(|f| is_unique(f)));
        }
    }

    if unique_selectors.is_empty() {
        return false;
    }

    complex_two.iter().any(|component| {
        if let ComplexSelectorComponent::Compound(compound) = component {
            compound
                .components
                .iter()
                .any(|simple| is_unique(simple) && unique_selectors.contains(&simple))
        } else {
            false
        }
    })
}

/// Returns whether a `CompoundSelector` may contain only one simple selector of
/// the same type as `simple`.
fn is_unique(simple: &SimpleSelector) -> bool {
    matches!(
        simple,
        SimpleSelector::Id(..)
            | SimpleSelector::Pseudo(Pseudo {
                is_class: false,
                ..
            })
    )
}
