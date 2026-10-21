use fervid_core::{SfcStyleBlock, fervid_atom};
use swc_html_ast::{Child, Element};

use crate::{ParseError, SfcParser, error::ParseErrorKind};

impl SfcParser<'_, '_, '_> {
    pub fn parse_sfc_style_element(&mut self, mut element: Element) -> Option<SfcStyleBlock> {
        // Find the attributes
        let mut lang = fervid_atom!("css");
        let mut is_scoped = false;
        let mut is_module = false;

        for attr in element.attributes.into_iter() {
            if attr.name.eq("lang") {
                let Some(attr_val) = attr.value else {
                    continue;
                };

                lang = attr_val;
            } else if attr.name.eq("scoped") {
                is_scoped = true;
            } else if attr.name.eq("module") {
                is_module = true;
            }
        }

        // `<style>` must have exactly one Text child
        let style_content = match element.children.pop() {
            Some(Child::Text(t)) => t,
            Some(_) => {
                self.report_error(ParseError {
                    kind: ParseErrorKind::UnexpectedNonRawTextContent,
                    span: element.span,
                });
                return None;
            }
            None if self.ignore_empty => {
                return None;
            }
            None => {
                return Some(SfcStyleBlock {
                    lang,
                    content: fervid_atom!(""),
                    is_scoped,
                    is_module,
                    span: element.span,
                });
            }
        };

        // Ignore empty unless allowed
        if self.ignore_empty && style_content.data.trim().is_empty() {
            return None;
        }

        Some(SfcStyleBlock {
            lang,
            content: style_content.data,
            is_scoped,
            is_module,
            span: style_content.span,
        })
    }
}
