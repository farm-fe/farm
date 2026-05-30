use farmfe_core::swc_common::{comments::Comments, Mark, SyntaxContext};
use farmfe_core::swc_ecma_ast::*;
use farmfe_toolkit::swc_atoms::Atom;
use fxhash::FxHashMap;
use std::collections::BTreeMap;

pub mod directive;
pub mod options;
pub mod patch_flags;
pub mod resolve_type;
pub mod slot_flag;
pub mod util;

pub struct VueJsxTransformVisitor<C>
where
    C: Comments,
{
    pub(crate) options: crate::options::Options,
    pub(crate) vue_imports: BTreeMap<&'static str, Ident>,
    pub(crate) interfaces: FxHashMap<(Atom, SyntaxContext), TsInterfaceDecl>,
    pub(crate) type_aliases: FxHashMap<(Atom, SyntaxContext), TsType>,
    pub(crate) unresolved_mark: Mark,
    pub(crate) comments: Option<C>,
}

impl<C> VueJsxTransformVisitor<C>
where
    C: Comments,
{
    pub(crate) fn import_from_vue(&mut self, item: &'static str) -> Ident {
        self.vue_imports
            .entry(item)
            .or_insert_with(|| {
                use farmfe_core::swc_common::DUMMY_SP;
                quote_ident_impl(item)
            })
            .clone()
    }
}

fn quote_ident_impl(s: &str) -> Ident {
    use farmfe_core::swc_common::DUMMY_SP;
    Ident {
        span: DUMMY_SP,
        sym: Atom::new(s),
        optional: false,
        ctxt: SyntaxContext::empty(),
    }
}
