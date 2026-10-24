use fervid_core::{AttributeOrBinding, FervidAtom, SfcCustomBlock, StartingTag, fervid_atom};
use swc_html_ast::Element;

use crate::SfcParser;

impl SfcParser<'_, '_, '_> {
    pub fn parse_sfc_custom_block_element(&mut self, element: Element) -> Option<SfcCustomBlock> {
        let attributes = element
            .attributes
            .into_iter()
            .map(|attr| AttributeOrBinding::RegularAttribute {
                name: attr.name,
                value: attr.value.unwrap_or_else(|| fervid_atom!("")),
                span: attr.span,
            })
            .collect();

        let (raw_content, _) =
            self.use_rawtext_content(element.content.as_ref(), &element.children)?;

        Some(SfcCustomBlock {
            starting_tag: StartingTag {
                tag_name: element.tag_name,
                attributes,
                directives: None,
            },
            content: FervidAtom::from(raw_content),
            span: element.span,
        })
    }
}
