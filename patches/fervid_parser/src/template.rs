use fervid_core::{
    AttributeOrBinding, ElementNode, FervidAtom, Interpolation, Node, PatchHints, SfcTemplateBlock,
    StartingTag, VueDirectives, fervid_atom, is_html_tag,
};
use swc_core::common::{BytePos, Span};
use swc_ecma_parser::{Syntax, TsSyntax};
use swc_html_ast::{Child, Element, Text};

use crate::SfcParser;

impl SfcParser<'_, '_, '_> {
    pub fn parse_template_to_ir(&mut self, root_element: Element) -> Option<SfcTemplateBlock> {
        // TODO Errors in template

        let lang_atom = fervid_atom!("lang");
        let html_atom = || fervid_atom!("html");

        let lang = root_element
            .attributes
            .into_iter()
            .find_map(|attr| {
                if attr.name == lang_atom {
                    Some(match attr.value {
                        Some(v) => {
                            let trimmed = v.trim();
                            if trimmed.is_empty() {
                                html_atom()
                            } else {
                                FervidAtom::from(trimmed)
                            }
                        }
                        None => html_atom(),
                    })
                } else {
                    None
                }
            })
            .unwrap_or_else(html_atom);

        // Do not parse non-html templates
        if lang != "html" {
            let roots = if let Some((content, content_span)) =
                self.use_rawtext_content(root_element.content.as_ref(), &root_element.children)
            {
                vec![Node::Text(FervidAtom::from(content), content_span)]
            } else {
                vec![]
            };

            return Some(SfcTemplateBlock {
                lang,
                roots,
                span: root_element.span,
            });
        }

        // <template> technically has a `content`
        let children: Vec<Child> = root_element
            .content
            .map(|c| c.children)
            .unwrap_or_else(|| root_element.children);

        Some(SfcTemplateBlock {
            lang,
            roots: self.process_element_children(children),
            span: root_element.span,
        })
    }

    fn process_element(&mut self, element: Element) -> Node {
        let children: Vec<Child> = element
            .content
            .map(|c| c.children)
            .unwrap_or_else(|| element.children);

        // For finer-grained `Span`s and components
        // TODO Add `tag_name_span` to `StartingTag`
        let raw_idx_start = element.span.lo.0 as usize;
        let raw_idx_end = raw_idx_start + element.tag_name.len();

        // Use raw names for custom HTML elements.
        // Otherwise SWC renames `CustomComponent` to `customcomponent`
        let tag_name = if is_html_tag(&element.tag_name) {
            element.tag_name
        } else {
            let raw_name = &self.input[raw_idx_start..raw_idx_end];
            raw_name.into()
        };

        // Save old `v-pre` (restored at the end of the function)
        let old_is_pre = self.is_pre;

        // Pre-allocate with excess, assuming all the attributes are not directives
        let mut attributes: Vec<AttributeOrBinding> = Vec::with_capacity(element.attributes.len());
        let mut directives: Option<Box<VueDirectives>> = None;

        // Process the attributes
        let has_v_pre =
            self.process_element_attributes(element.attributes, &mut attributes, &mut directives);

        // Add an indicator directive for `v-pre`
        if has_v_pre {
            let directives = directives.get_or_insert_with(|| Box::new(VueDirectives::default()));
            directives.v_pre = Some(());
            self.is_pre = true;
        }

        let starting_tag = StartingTag {
            tag_name,
            attributes,
            directives,
        };

        let result = Node::Element(ElementNode {
            kind: fervid_core::ElementKind::Element,
            starting_tag,
            children: self.process_element_children(children),
            template_scope: 0,
            patch_hints: PatchHints::default(),
            span: element.span,
        });

        self.is_pre = old_is_pre;
        result
    }

    fn process_element_children(&mut self, children: Vec<Child>) -> Vec<Node> {
        let mut out = Vec::with_capacity(children.len());

        for child in children {
            match child {
                Child::DocumentType(_) => unimplemented!("Doctype is unsupported"),
                Child::Element(element) => out.push(self.process_element(element)),
                Child::Text(text) => self.process_text(text, &mut out),
                Child::Comment(comment) => out.push(Node::Comment(comment.data, comment.span)),
            }
        }

        out
    }

    /// Separates a raw text into `Node::Text`s and `Node::Interpolation`s
    fn process_text(&mut self, text: Text, out: &mut Vec<Node>) {
        // `v-pre` logic
        if self.is_pre {
            out.push(Node::Text(text.data, text.span));
            return;
        }

        let interpolation_start_pat = self.interpolation_start_pat;
        let interpolation_end_pat = self.interpolation_end_pat;
        let Text { span, data, .. } = text;
        let raw: &str = &data;
        let interpolation_start_pat_len = interpolation_start_pat.len();
        let interpolation_end_pat_len = interpolation_end_pat.len();

        // let mut curr_text = "";
        // let mut curr_text_start_idx = 0;
        // let mut curr_text_end_idx = 0;
        let mut text_start_idx = 0;

        // Find interpolation start - `{{` by default
        for (match_idx, _) in raw.match_indices(interpolation_start_pat) {
            let interpolation_start_idx = match_idx + interpolation_start_pat_len;

            // Find interpolation end - `}}` by default
            let Some(interpolation_end_idx) =
                raw[interpolation_start_idx..].find(interpolation_end_pat)
            else {
                continue;
            };

            // Offset, because we did offset while `find`ing previously
            let interpolation_end_idx = interpolation_end_idx + interpolation_start_idx;

            // Add any previous text
            if text_start_idx < match_idx {
                let offset = span.lo.0 + text_start_idx as u32;
                let text = &raw[text_start_idx..match_idx];
                let text_span = Span {
                    lo: BytePos(offset),
                    hi: BytePos(offset + text.len() as u32),
                };

                out.push(Node::Text(FervidAtom::from(text), text_span));
            }
            text_start_idx = interpolation_end_idx + interpolation_end_pat_len;

            // Get the interpolation &str
            let interpolation = &raw[interpolation_start_idx..interpolation_end_idx];

            // Span stuff
            let offset = span.lo.0 + interpolation_start_idx as u32;
            let interpolation_span = Span::new(
                BytePos(offset),
                BytePos(offset + interpolation.len() as u32),
            );

            match self.parse_expr(
                interpolation,
                Syntax::Typescript(TsSyntax::default()),
                interpolation_span,
            ) {
                Ok(parsed_interpolation) => out.push(Node::Interpolation(Interpolation {
                    value: parsed_interpolation,
                    template_scope: 0,
                    patch_flag: false,
                    span: interpolation_span,
                })),
                Err(expr_err) => self.report_error(expr_err),
            }
        }

        // Add the remaining text if any
        if text_start_idx < raw.len() {
            let text_span = Span::new(BytePos(span.lo.0 + text_start_idx as u32), span.hi);
            out.push(Node::Text(
                FervidAtom::from(&raw[text_start_idx..]),
                text_span,
            ));
        }

        // let mut remaining = raw;
        // let mut remaining_start_idx = 0;

        // // Find interpolation start - `{{` by default
        // for (match_idx, _) in raw.match_indices(interpolation_start_pat) {
        //     let interpolation_start_idx = match_idx + interpolation_start_pat_len;

        //     // Find interpolation end - `}}` by default
        //     if let Some(interpolation_end_idx) =
        //         raw[interpolation_start_idx..].find(interpolation_end_pat)
        //     {
        //         // Offset
        //         let interpolation_end_idx = interpolation_end_idx + interpolation_start_idx;
        //         let interpolation = &raw[interpolation_start_idx..interpolation_end_idx];

        //         // Include everything prior to interpolation

        //         // Span stuff
        //         let offset = span.lo.0 + interpolation_start_idx as u32;
        //         let interpolation_span = Span::new(
        //             BytePos(offset),
        //             BytePos(offset + interpolation.len() as u32),
        //             Default::default(),
        //         );

        //         match parse_expr(
        //             interpolation,
        //             Syntax::Typescript(TsConfig::default()),
        //             interpolation_span,
        //         ) {
        //             Ok(parsed_interpolation) => out.push(Node::Interpolation(Interpolation {
        //                 value: parsed_interpolation,
        //                 template_scope: 0,
        //                 patch_flag: false,
        //             })),
        //             Err(expr_err) => errors.push(expr_err.into()),
        //         }

        //         // Advance
        //         remaining_start_idx = interpolation_end_idx + interpolation_end_pat_len;
        //         remaining = &raw[remaining_start_idx..];
        //     }
        // }

        // if !remaining.is_empty() {
        //     let offset = span.lo.0 + remaining_start_idx as u32;
        //     let new_span = Span::new(
        //         BytePos(offset),
        //         BytePos(offset + remaining.len() as u32),
        //         Default::default(),
        //     );
        //     out.push(Node::Text(FervidAtom::from(remaining), new_span));
        // }
    }
}

#[cfg(test)]
mod tests {
    use swc_core::ecma::ast::Expr;

    use super::*;

    #[test]
    fn it_acknowledges_v_pre() {
        let mut errors = Vec::new();
        let mut parser = SfcParser::new(
            r#"
        <template><div :parse-this="false" v-pre><h1>  {{ msg }}  </h1><input v-model="msg" v-pre></div></template>"#,
            &mut errors,
        );

        let parsed = parser.parse_sfc().expect("Should parse");
        let template = parsed.template.expect("Should have template");
        let first_root = template.roots.first().expect("Should have one root");
        let Node::Element(first_div) = first_root else {
            panic!("Root is not an element")
        };

        // Check the presence of a directive
        let first_div_directives = first_div
            .starting_tag
            .directives
            .as_ref()
            .expect("Should have directives");
        assert!(first_div_directives.v_pre.is_some());

        // Check that exactly one attribute is there, and it is regular (not parsed)
        assert_eq!(1, first_div.starting_tag.attributes.len());
        assert!(matches!(
            first_div.starting_tag.attributes.first(),
            Some(AttributeOrBinding::RegularAttribute { name, value, .. }) if name == ":parse-this" && value == "false"
        ));

        // Check the <h1>
        let Some(Node::Element(h1)) = first_div.children.first() else {
            panic!("First child of div is not h1")
        };
        assert!(h1.starting_tag.attributes.is_empty());
        assert_eq!(1, h1.children.len());
        let Some(Node::Text(h1_text, _)) = h1.children.first() else {
            panic!("First child of h1 is not text")
        };
        assert!(h1_text.trim() == "{{ msg }}");

        // Check the <input>
        let Some(Node::Element(input)) = first_div.children.last() else {
            panic!("Last child of div is not input")
        };
        assert!(input.starting_tag.directives.is_none());
        assert_eq!(2, input.starting_tag.attributes.len());

        // Check the <input> attrs
        let input_attr_1 = input.starting_tag.attributes.first().unwrap();
        let input_attr_2 = input.starting_tag.attributes.get(1).unwrap();
        assert!(
            matches!(input_attr_1, AttributeOrBinding::RegularAttribute { name, value, .. } if name == "v-model" && value == "msg")
        );
        assert!(
            // Second v-pre should not be recognized as a directive, because it is inside the first
            matches!(input_attr_2, AttributeOrBinding::RegularAttribute { name, value, .. } if name == "v-pre" && value.is_empty())
        );
    }

    #[test]
    fn it_resets_state_after_v_pre() {
        let mut errors = Vec::new();
        let mut parser = SfcParser::new(
            r#"
        <template><div v-pre></div><h1>{{ msg }}</h1></template>"#,
            &mut errors,
        );

        let parsed = parser.parse_sfc().expect("Should parse");
        let template = parsed.template.expect("Should have template");

        // Check div
        let Some(Node::Element(div)) = template.roots.first() else {
            panic!("First child of root is not div")
        };
        let div_directives = div
            .starting_tag
            .directives
            .as_ref()
            .expect("Should have directives");
        assert!(div_directives.v_pre.is_some());

        // Check h1
        let Some(Node::Element(h1)) = template.roots.last() else {
            panic!("Last child of root is not h1")
        };
        assert!(h1.starting_tag.directives.is_none());
        assert_eq!(1, h1.children.len());

        // Check h1 child
        let Some(Node::Interpolation(interpolation)) = h1.children.first() else {
            panic!("First child of h1 is not interpolation")
        };
        assert!(interpolation.value.is_ident());
    }

    #[test]
    fn it_supports_shorthand_v_bind() {
        let mut errors = Vec::new();
        let mut parser = SfcParser::new(
            r#"
        <template><div :foo-bar-baz></div></template>"#,
            &mut errors,
        );

        let parsed = parser.parse_sfc().expect("Should parse");
        let template = parsed.template.expect("Should have template");

        // Check div
        let Some(Node::Element(div)) = template.roots.first() else {
            panic!("First child of root is not div")
        };

        // Check directive
        let Some(AttributeOrBinding::VBind(v_bind)) = div.starting_tag.attributes.first() else {
            panic!("v-bind not found")
        };
        assert!(matches!(v_bind.value.as_ref(), Expr::Ident(ident) if ident.sym == "fooBarBaz"));
    }
}
