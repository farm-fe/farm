mod attributes;
mod custom_block;
mod error;
mod script;
mod sfc;
mod style;
mod template;

pub use error::ParseError;
use swc_core::common::comments::SingleThreadedComments;

// Default patterns for interpolation
pub const INTERPOLATION_START_PAT_DEFAULT: &str = "{{";
pub const INTERPOLATION_END_PAT_DEFAULT: &str = "}}";

#[derive(Debug)]
pub struct SfcParser<'i, 'e, 'p> {
    input: &'i str,
    errors: &'e mut Vec<ParseError>,
    comments: SingleThreadedComments,
    is_pre: bool,
    interpolation_start_pat: &'p str,
    interpolation_end_pat: &'p str,
    pub ignore_empty: bool,
}

impl<'i, 'e> SfcParser<'i, 'e, 'static> {
    pub fn new(input: &'i str, errors: &'e mut Vec<ParseError>) -> Self {
        // TODO When should it fail? What do we do with errors?..
        // I was thinking of 4 strategies:
        // `HARD_FAIL_ON_ERROR` (any error = fail (incl. recoverable), stop at the first error),
        // `SOFT_REPORT_ALL` (any error = fail (incl. recoverable), continue as far as possible and report after),
        // `SOFT_RECOVER_SAFE` (try to ignore recoverable, report the rest),
        // `SOFT_RECOVER_UNSAFE` (ignore as much as is possible, but still report).

        SfcParser {
            input,
            errors,
            comments: SingleThreadedComments::default(),
            is_pre: false,
            interpolation_start_pat: INTERPOLATION_START_PAT_DEFAULT,
            interpolation_end_pat: INTERPOLATION_END_PAT_DEFAULT,
            ignore_empty: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use fervid_core::{Node, SfcDescriptor, SfcScriptLang};
    use swc_core::ecma::ast::{ModuleDecl, ModuleItem};

    use crate::{ParseError, SfcParser, error::ParseErrorKind};

    const SHOULD_EXIST: &str = "Should exist";

    #[test]
    fn it_works() {
        let document = include_str!("../tests/fixtures/input.vue");

        let mut errors = Vec::new();

        // Cold
        let now = std::time::Instant::now();
        let mut parser = SfcParser::new(document, &mut errors);
        let _ = parser.parse_sfc();
        let elapsed = now.elapsed();
        println!("Elapsed: {:?}", elapsed);
        parser.errors.clear();

        // Hot
        let now = std::time::Instant::now();
        let parsed = parser.parse_sfc();
        let elapsed = now.elapsed();
        println!("Elapsed: {:?}", elapsed);

        println!("{:#?}", parsed);

        for error in errors {
            let e = error;
            println!(
                "{:?} {}",
                e.kind,
                &document[e.span.lo.0 as usize - 1..e.span.hi.0 as usize - 1]
            );
        }

        // assert_eq!(errors.len(), 0);
    }

    // Tests below are adapted from
    // https://github.com/vuejs/core/blob/a41c5f1f4367a9f41bcdb8c4e02f54b2378e577d/packages/compiler-sfc/__tests__/parse.spec.ts

    #[test]
    fn style_block() {
        let (mut src, _) = padding();
        src.push_str(
            r"<style>
.css {
color: red;
}
</style>

<style module>
.css-module {
color: red;
}
</style>

<style scoped>
.css-scoped {
color: red;
}
</style>

<style scoped>
.css-scoped-nested {
color: red;
.dummy {
color: green;
}
font-weight: bold;
}
</style>",
        );

        let styles = parse(&src).styles;
        assert_eq!(4, styles.len());
        assert!(styles[0].lang == "css" && !styles[0].is_scoped && !styles[0].is_module);
        assert!(styles[1].lang == "css" && !styles[1].is_scoped && styles[1].is_module);
        assert!(styles[2].lang == "css" && styles[2].is_scoped && !styles[2].is_module);
        assert!(styles[3].lang == "css" && styles[3].is_scoped && !styles[3].is_module);
    }

    #[test]
    fn script_block() {
        let (mut src, _) = padding();
        // The original example is a syntax error
        // <script>\nconsole.log(1)\n }\n</script>\n
        src.push_str("<script>\nconsole.log(1)\n \n</script>\n");

        let script = parse(&src).script_legacy.expect(SHOULD_EXIST);
        assert!(matches!(script.lang, SfcScriptLang::Es));
    }

    #[test]
    fn template_block_with_lang_and_indent() {
        let (mut src, _) = padding();
        src.push_str(
            r#"<template lang="pug">
  h1 foo
    div bar
    span baz
</template>\n"#,
        );

        let template = parse(&src).template.expect(SHOULD_EXIST);
        assert!(template.lang == "pug");
    }

    #[test]
    fn custom_block() {
        let (mut src, _) = padding();
        src.push_str(r#"<i18n>\n{\n  "greeting": "hello"\n}\n</i18n>\n"#);

        let custom_blocks = parse(&src).custom_blocks;
        assert_eq!(1, custom_blocks.len());
        assert!(custom_blocks[0].starting_tag.tag_name == "i18n");
        assert!(!custom_blocks[0].content.is_empty());
    }

    #[test]
    fn pad_content() {
        let descriptor = parse(
            r#"
<template>
<div></div>
</template>
<script>
export default {}
</script>
<style>
h1 { color: red }
</style>
<i18n>
{ "greeting": "hello" }
</i18n>
        "#,
        );

        let template = descriptor.template.expect(SHOULD_EXIST);
        assert_eq!(3, template.roots.len()); // whitespace around div
        assert!(matches!(template.roots[1], Node::Element(_)));

        let script = descriptor.script_legacy.expect(SHOULD_EXIST);
        assert_eq!(1, script.content.body.len());
        assert!(matches!(
            script.content.body[0],
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(_))
        ));

        let styles = descriptor.styles;
        assert_eq!(1, styles.len());
        let style = &styles[0];
        assert!(style.content == "\nh1 { color: red }\n");

        let custom_blocks = descriptor.custom_blocks;
        assert_eq!(1, custom_blocks.len());
        let custom_block = &custom_blocks[0];
        dbg!(&custom_block.content);
        assert!(custom_block.content == "\n{ \"greeting\": \"hello\" }\n");

        // `pad: true` is not supported
    }

    #[test]
    fn should_parse_correct_range_for_root_level_self_closing_tag() {
        let content = "\n  <div/>\n";
        let template = parse(&format!("<template>{content}</template>"))
            .template
            .expect(SHOULD_EXIST);

        assert_eq!(3, template.roots.len());

        let Node::Element(div) = &template.roots[1] else {
            panic!("Expected element");
        };
        assert_eq!(14, div.span.lo.0);
        assert_eq!(14 + content.trim().len() as u32, div.span.hi.0);
    }

    #[test]
    fn should_parse_correct_range_for_blocks_with_no_content_self_closing() {
        let template = parse("<template/>").template.expect(SHOULD_EXIST);
        assert!(template.roots.is_empty());
        assert_eq!(1, template.span.lo.0);
        assert_eq!(12, template.span.hi.0);
    }

    #[test]
    fn should_parse_correct_range_for_blocks_with_no_content_explicit() {
        let template = parse("<template></template>").template.expect(SHOULD_EXIST);
        assert!(template.roots.is_empty());
        assert_eq!(1, template.span.lo.0);
        assert_eq!(22, template.span.hi.0);
    }

    #[test]
    fn should_ignore_other_nodes_with_no_content() {
        assert!(parse("<script/>").script_legacy.is_none());
        assert!(parse("<script> \n\t  </script>").script_legacy.is_none());
        assert!(parse("<style/>").styles.is_empty());
        assert!(parse("<style> \n\t </style>").styles.is_empty());
        assert!(parse("<custom/>").custom_blocks.is_empty());
        assert!(parse("<custom> \n\t </custom>").custom_blocks.is_empty());
    }

    #[test]
    fn handle_empty_nodes_with_src_attribute() {
        // src imports not supported
        // supporting them forces fervid to make assumptions about the environment
        // they can be supported via a FileSystem adapter
        // <script src="com"/>
    }

    #[test]
    fn should_not_expose_ast_on_template_node_if_has_src_import() {
        // src imports not supported
        // supporting them forces fervid to make assumptions about the environment
        // they can be supported via a FileSystem adapter
        // <template src="./foo.html"/>
    }

    #[test]
    fn ignore_empty_false() {
        let mut errors = Vec::new();
        let mut parser =
            SfcParser::new("<script></script>\n<script setup>\n</script>", &mut errors);
        parser.ignore_empty = false;
        let descriptor = parser.parse_sfc().unwrap();

        let script_legacy = descriptor.script_legacy.expect(SHOULD_EXIST);
        assert_eq!(1, script_legacy.span.lo.0);
        assert_eq!(18, script_legacy.span.hi.0);

        let script_setup = descriptor.script_setup.expect(SHOULD_EXIST);
        assert_eq!(19, script_setup.span.lo.0);
        assert_eq!(43, script_setup.span.hi.0);
    }

    #[test]
    fn nested_templates() {
        let template = parse(
            r#"<template>
                <template v-if="ok">ok</template>
                <div><div></div></div>
            </template>"#,
        )
        .template
        .expect(SHOULD_EXIST);

        assert_eq!(5, template.roots.len());
        assert!(matches!(template.roots[0], Node::Text(_, _)));
        assert!(matches!(template.roots[1], Node::Element(_)));
        assert!(matches!(template.roots[2], Node::Text(_, _)));
        assert!(matches!(template.roots[3], Node::Element(_)));
        assert!(matches!(template.roots[4], Node::Text(_, _)));
    }

    #[test]
    fn treat_empty_lang_attribute_as_the_html() {
        let template =
            parse(r#"<template lang=""><div><template v-if="ok">ok</template></div></template>"#)
                .template
                .expect(SHOULD_EXIST);
        assert!(template.lang == "html");
        assert_eq!(1, template.roots.len());
    }

    #[test]
    fn template_with_preprocessor_lang_should_be_treated_as_plain_text() {
        let content = r#"p(v-if="1 < 2") test <div/>"#;
        let source = format!("<template lang=\"pug\">{content}</template>");

        let (descriptor, errors) = parse_with_errors(&source);
        assert!(errors.is_empty());

        let template = descriptor.template.expect(SHOULD_EXIST);
        assert_eq!(1, template.roots.len());
        dbg!(&template.roots);
        assert!(matches!(&template.roots[0], Node::Text(t, _) if t == content));
    }

    #[test]
    fn div_lang_should_not_be_treated_as_plain_text() {
        let (_, errors) = parse_with_errors(
            r#"<template lang="pug">
        <div lang="">
          <div></div>
        </div>
      </template>"#,
        );

        assert!(errors.is_empty());
    }

    #[test]
    fn slotted_detection() {
        // TODO Implement :slotted and ::v-slotted detection
        let _descriptor1 = parse("<template>hi</template>");
        let _descriptor2 = parse("<template>hi</template><style>h1{color:red;}</style>");
        let _descriptor3 =
            parse("<template>hi</template><style scoped>:slotted(h1){color:red;}</style>");
        let _descriptor4 =
            parse("<template>hi</template><style scoped>::v-slotted(h1){color:red;}</style>");
    }

    #[test]
    fn error_tolerance() {
        let (_, errors) = parse_with_errors("<template>");
        assert_eq!(1, errors.len());
    }

    #[test]
    fn should_parse_as_dom_by_default() {
        let (_, errors) = parse_with_errors("<template><input></template>");
        assert!(errors.is_empty());
    }

    #[test]
    fn treat_custom_blocks_as_raw_text() {
        let (descriptor, errors) =
            parse_with_errors("<template><input></template><foo> <-& </foo>");
        assert!(errors.is_empty());
        assert_eq!(1, descriptor.custom_blocks.len());
        assert!(descriptor.custom_blocks[0].content == " <-& ");
    }

    #[test]
    fn should_only_allow_single_template_element() {
        let (_, errors) =
            parse_with_errors("<template><div/></template><template><div/></template>");
        assert!(
            errors
                .iter()
                .any(|e| matches!(&e.kind, ParseErrorKind::DuplicateTemplate))
        );
    }

    #[test]
    fn should_only_allow_single_script_element() {
        let (_, errors) =
            parse_with_errors("<script>console.log(1)</script><script>console.log(1)</script>");
        assert!(
            errors
                .iter()
                .any(|e| matches!(&e.kind, ParseErrorKind::DuplicateScriptOptions))
        );
    }

    #[test]
    fn should_only_allow_single_script_setup_element() {
        let (_, errors) = parse_with_errors(
            "<script setup>console.log(1)</script><script setup>console.log(1)</script>",
        );
        assert!(
            errors
                .iter()
                .any(|e| matches!(&e.kind, ParseErrorKind::DuplicateScriptSetup))
        );
    }

    #[test]
    fn should_not_warn_script_script_setup() {
        let (_, errors) = parse_with_errors(
            "<script setup>console.log(1)</script><script>console.log(1)</script>",
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn should_throw_error_if_no_template_or_script_is_present() {
        let (_, errors) = parse_with_errors("import { ref } from 'vue'");
        assert!(
            errors
                .iter()
                .any(|e| matches!(&e.kind, ParseErrorKind::MissingTemplateOrScript))
        );
    }

    fn parse(source: &str) -> SfcDescriptor {
        let mut errors = Vec::new();
        let mut parser = SfcParser::new(source, &mut errors);
        // dbg!(parser.errors);
        parser.parse_sfc().unwrap()
    }

    fn parse_with_errors(source: &str) -> (SfcDescriptor, Vec<ParseError>) {
        let mut errors = Vec::new();
        let mut parser = SfcParser::new(source, &mut errors);
        let descriptor = parser.parse_sfc().unwrap();
        let errors = std::mem::take(parser.errors);
        (descriptor, errors)
    }

    fn padding() -> (String, usize) {
        // No random
        let padding = 4;
        ("\n".repeat(padding), padding)
    }
}
