use swc_core::common::{Span, Spanned};

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    /// Malformed directive (e.g. `:`, `@`)
    DirectiveSyntax,
    /// Malformed directive name (e.g. `v-.`)
    DirectiveSyntaxDirectiveName,
    /// Malformed directive argument (e.g. `v-bind:`)
    DirectiveSyntaxArgument,
    /// Malformed directive dynamic argument (e.g. `:[foo`)
    DirectiveSyntaxDynamicArgument,
    /// Malformed directive - expected modifiers, but got something else (e.g. `:[foo]v`)
    DirectiveSyntaxUnexpectedCharacterAfterDynamicArgument,
    /// Malformed directive modifier (e.g. `:smth.`)
    DirectiveSyntaxModifier,
    /// More than one `<script>`
    DuplicateScriptOptions,
    /// More than one `<script setup>`
    DuplicateScriptSetup,
    /// More than one `<template>`
    DuplicateTemplate,
    /// More than one attribute with the same name on a root element
    DuplicateAttribute,
    /// Error while parsing EcmaScript/TypeScript
    EcmaSyntaxError(Box<swc_ecma_parser::error::SyntaxError>),
    /// Unrecoverable error while parsing HTML
    InvalidHtml(Box<swc_html_parser::error::ErrorKind>),
    /// Both `<template>` and `<script>` are missing
    MissingTemplateOrScript,
    /// `<script>`/`<style>` content was not Text
    UnexpectedNonRawTextContent,
    /// Language not supported
    UnsupportedLang,
}

impl From<swc_ecma_parser::error::Error> for ParseError {
    fn from(value: swc_ecma_parser::error::Error) -> ParseError {
        let span = value.span();

        ParseError {
            kind: ParseErrorKind::EcmaSyntaxError(Box::new(value.into_kind())),
            span,
        }
    }
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Spanned for ParseError {
    fn span(&self) -> Span {
        self.span
    }
}
