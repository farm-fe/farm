use fervid_core::{FervidAtom, SfcDescriptor, fervid_atom};
use swc_core::common::{BytePos, DUMMY_SP, Span, Spanned};
use swc_ecma_parser::StringInput;
use swc_html_ast::{
    Child, DocumentFragment, DocumentMode, Element, Namespace, Token, TokenAndSpan,
};
use swc_html_parser::{
    error::ErrorKind,
    lexer::{Lexer, State},
    parser::{Parser, ParserConfig, input::ParserInput},
};

use crate::{
    SfcParser,
    error::{ParseError, ParseErrorKind},
};

type SwcHtmlParserError = swc_html_parser::error::Error;

// HTML tokenization lowercases tag names. Vue's PascalCase components must
// remain distinct before tree construction: <Title/> must not enter RCDATA,
// and <Button> must not trigger the native button insertion rules.
struct VueComponentInput<I>(I);

impl<I: ParserInput> Iterator for VueComponentInput<I> {
    type Item = TokenAndSpan;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = self.0.next()?;
        match &mut item.token {
            Token::StartTag {
                tag_name,
                raw_tag_name,
                ..
            }
            | Token::EndTag {
                tag_name,
                raw_tag_name,
                ..
            } => {
                if let Some(raw) = raw_tag_name {
                    if raw.as_bytes().first().is_some_and(u8::is_ascii_uppercase) {
                        *tag_name = raw.clone();
                    }
                }
            }
            _ => {}
        }
        Some(item)
    }
}

impl<I: ParserInput> ParserInput for VueComponentInput<I> {
    fn start_pos(&mut self) -> BytePos {
        self.0.start_pos()
    }
    fn last_pos(&mut self) -> BytePos {
        self.0.last_pos()
    }
    fn take_errors(&mut self) -> Vec<SwcHtmlParserError> {
        self.0.take_errors()
    }
    fn set_last_start_tag_name(&mut self, name: &FervidAtom) {
        self.0.set_last_start_tag_name(name);
    }
    fn set_input_state(&mut self, state: State) {
        self.0.set_input_state(state);
    }
    fn set_adjusted_current_node_to_html_namespace(&mut self, value: bool) {
        self.0.set_adjusted_current_node_to_html_namespace(value);
    }
}

impl SfcParser<'_, '_, '_> {
    /// Parses `self.input` as an SFC, producing an `SfcDescriptor`.
    /// When `Err(ParseError)` is returned, that means unrecoverable error was discovered.
    pub fn parse_sfc(&mut self) -> Result<SfcDescriptor, ParseError> {
        let parsed_html = self.parse_html_document_fragment().map_err(|e| {
            let kind = e.into_inner().1;

            ParseError {
                kind: ParseErrorKind::InvalidHtml(Box::new(kind)),
                span: Span {
                    lo: BytePos(1),
                    hi: BytePos(self.input.len() as u32),
                },
            }
        })?;

        let mut sfc_descriptor = SfcDescriptor::default();

        macro_rules! report_error {
            ($kind: ident, $span: expr) => {
                self.report_error(ParseError {
                    kind: ParseErrorKind::$kind,
                    span: $span,
                });
            };
        }

        for root_node in parsed_html.children.into_iter() {
            // Only root elements are supported
            let Child::Element(root_element) = root_node else {
                continue;
            };

            let tag_name = &root_element.tag_name;
            let root_node_span = root_element.span;

            if tag_name.eq("template") {
                // Check duplicate
                if sfc_descriptor.template.is_some() {
                    report_error!(DuplicateTemplate, root_node_span);
                    continue;
                }

                let template_result = self.parse_template_to_ir(root_element);
                if template_result.is_none() {
                    // TODO Error
                    continue;
                };

                sfc_descriptor.template = template_result;
            } else if tag_name.eq("script") {
                let sfc_script_block = match self.parse_sfc_script_element(root_element) {
                    Ok(Some(v)) => v,
                    Ok(None) => continue,
                    Err(e) => {
                        self.report_error(e);
                        continue;
                    }
                };

                if sfc_script_block.is_setup {
                    // Check if already present
                    if sfc_descriptor.script_setup.is_some() {
                        report_error!(DuplicateScriptSetup, root_node_span);
                        continue;
                    }
                    sfc_descriptor.script_setup = Some(sfc_script_block);
                } else {
                    // Check if already present
                    if sfc_descriptor.script_legacy.is_some() {
                        report_error!(DuplicateScriptOptions, root_node_span);
                        continue;
                    }
                    sfc_descriptor.script_legacy = Some(sfc_script_block);
                }
            } else if tag_name.eq("style") {
                if let Some(style_block) = self.parse_sfc_style_element(root_element) {
                    sfc_descriptor.styles.push(style_block);
                }
            } else if let Some(custom_block) = self.parse_sfc_custom_block_element(root_element) {
                sfc_descriptor.custom_blocks.push(custom_block);
            }
        }

        // Emit an error if neither of `<template>` and both `<script>`s are present
        if sfc_descriptor.template.is_none()
            && sfc_descriptor.script_legacy.is_none()
            && sfc_descriptor.script_setup.is_none()
        {
            self.report_error(ParseError {
                kind: ParseErrorKind::MissingTemplateOrScript,
                span: parsed_html.span,
            });
        }

        // Ignore NonVoidHtmlElementStartTagWithTrailingSolidus because this is a difference of Vue SFC vs HTML
        self.errors.retain(|parse_error| {
            !matches!(
                parse_error.kind,
                ParseErrorKind::InvalidHtml(ref invalid_html_error)
                if matches!(invalid_html_error.as_ref(), ErrorKind::NonVoidHtmlElementStartTagWithTrailingSolidus)
            )
        });

        Ok(sfc_descriptor)
    }

    /// Adapted from `swc_html_parser`
    #[inline]
    pub fn parse_html_document_fragment(&mut self) -> Result<DocumentFragment, SwcHtmlParserError> {
        let lexer = Lexer::new(StringInput::new(
            self.input,
            BytePos(1),
            BytePos(self.input.len() as u32),
        ));

        let parser_config = ParserConfig {
            scripting_enabled: false,
            iframe_srcdoc: false,
            allow_self_closing: true,
        };
        let mut parser = Parser::new(VueComponentInput(lexer), parser_config);

        let ctx_element = Element {
            span: DUMMY_SP,
            tag_name: fervid_atom!("div"),
            namespace: Namespace::HTML,
            attributes: vec![],
            children: vec![],
            content: None,
            is_self_closing: false,
        };

        let result = parser.parse_document_fragment(ctx_element, DocumentMode::NoQuirks, None);

        let html_parse_errors = parser.take_errors();

        // TODO Check for serious errors (because it may parse regardless) ???
        self.errors.reserve(html_parse_errors.len());
        for html_parse_error in html_parse_errors {
            let e = html_parse_error.into_inner();
            self.errors.push(ParseError {
                kind: ParseErrorKind::InvalidHtml(Box::new(e.1)),
                span: e.0,
            })
        }

        result
    }

    /// Gets the raw contents of Element and also clears errors related to parsing it
    pub fn use_rawtext_content(
        &mut self,
        element_content: Option<&DocumentFragment>,
        element_children: &[Child],
    ) -> Option<(&str, Span)> {
        // For DocumentFragment, use its children.
        // Otherwise, because DocumentFragment is content of <template>,
        // it has the exact same span.
        let children = if let Some(content) = element_content {
            &content.children
        } else {
            element_children
        };

        // Span via first and last children
        let content_span = if let (Some(first), Some(last)) = (children.first(), children.last()) {
            Some(Span {
                lo: first.span_lo(),
                hi: last.span_hi(),
            })
        } else {
            None
        };

        // Get raw content
        let raw = if let Some(span) = content_span {
            &self.input[(span.lo.0 - 1) as usize..(span.hi.0 - 1) as usize]
        } else {
            ""
        };

        // Ignore empty unless allowed
        if self.ignore_empty && raw.trim().is_empty() {
            return None;
        }

        // Ignore swc errors occurring in the rawtext span
        if let Some(span) = content_span {
            self.errors.retain(|e| !span.contains(e.span));
        }

        // Dummy is returned when `ignore_empty = false` and content is empty
        // It would be better to return span between `<></>` with lo=hi,
        // but SWC only provides spans including tags and I don't want to guess.
        Some((raw, content_span.unwrap_or(DUMMY_SP)))
    }

    #[inline]
    pub fn report_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }
}
