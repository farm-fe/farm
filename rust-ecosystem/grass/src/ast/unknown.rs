use crate::ast::CssStmt;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct UnknownAtRule {
    pub name: String,
    // pub super_selector: Selector,
    pub params: String,
    pub body: Vec<CssStmt>,

    /// Whether or not this @-rule was declared with curly
    /// braces. A body may not necessarily have contents
    pub has_body: bool,
}
