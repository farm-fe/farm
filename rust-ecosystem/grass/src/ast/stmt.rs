use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use codemap::{Span, Spanned};

use crate::{
    ast::{ArgumentDeclaration, ArgumentInvocation, AstExpr, CssStmt},
    ast::{Interpolation, MediaQuery},
    common::Identifier,
    utils::{BaseMapView, LimitedMapView, MapView, UnprefixedMapView},
    value::Value,
};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct AstSilentComment {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstPlainCssImport {
    pub url: Interpolation,
    pub modifiers: Option<Interpolation>,
    #[allow(unused)]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstSassImport {
    pub url: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstIf {
    pub if_clauses: Vec<AstIfClause>,
    pub else_clause: Option<Vec<AstStmt>>,
}

#[derive(Debug, Clone)]
pub struct AstIfClause {
    pub condition: AstExpr,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone)]
pub struct AstFor {
    pub variable: Spanned<Identifier>,
    pub from: Spanned<AstExpr>,
    pub to: Spanned<AstExpr>,
    pub is_exclusive: bool,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone)]
pub struct AstReturn {
    pub val: AstExpr,
    #[allow(unused)]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstRuleSet {
    pub selector: Interpolation,
    pub body: Vec<AstStmt>,
    pub selector_span: Span,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstStyle {
    pub name: Interpolation,
    pub value: Option<Spanned<AstExpr>>,
    pub body: Vec<AstStmt>,
    pub span: Span,
}

impl AstStyle {
    pub fn is_custom_property(&self) -> bool {
        self.name.initial_plain().starts_with("--")
    }
}

#[derive(Debug, Clone)]
pub struct AstEach {
    pub variables: Vec<Identifier>,
    pub list: AstExpr,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone)]
pub struct AstMedia {
    pub query: Interpolation,
    pub query_span: Span,
    pub body: Vec<AstStmt>,
    pub span: Span,
}

pub type CssMediaQuery = MediaQuery;

#[derive(Debug, Clone)]
pub struct AstWhile {
    pub condition: AstExpr,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone)]
pub struct AstVariableDecl {
    pub namespace: Option<Spanned<Identifier>>,
    pub name: Identifier,
    pub value: AstExpr,
    pub is_guarded: bool,
    pub is_global: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstFunctionDecl {
    pub name: Spanned<Identifier>,
    pub arguments: ArgumentDeclaration,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone)]
pub struct AstDebugRule {
    pub value: AstExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstWarn {
    pub value: AstExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstErrorRule {
    pub value: AstExpr,
    pub span: Span,
}

impl PartialEq for AstFunctionDecl {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for AstFunctionDecl {}

#[derive(Debug, Clone)]
pub struct AstLoudComment {
    pub text: Interpolation,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstMixin {
    pub name: Identifier,
    pub args: ArgumentDeclaration,
    pub body: Vec<AstStmt>,
    /// Whether the mixin contains a `@content` rule.
    pub has_content: bool,
}

#[derive(Debug, Clone)]
pub struct AstContentRule {
    pub args: ArgumentInvocation,
}

#[derive(Debug, Clone)]
pub struct AstContentBlock {
    pub args: ArgumentDeclaration,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone)]
pub struct AstInclude {
    pub namespace: Option<Spanned<Identifier>>,
    pub name: Spanned<Identifier>,
    pub args: ArgumentInvocation,
    pub content: Option<AstContentBlock>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstUnknownAtRule {
    pub name: Interpolation,
    pub value: Option<Interpolation>,
    pub body: Option<Vec<AstStmt>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstExtendRule {
    pub value: Interpolation,
    pub is_optional: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstAtRootRule {
    pub body: Vec<AstStmt>,
    pub query: Option<Spanned<Interpolation>>,
    #[allow(unused)]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AtRootQuery {
    pub include: bool,
    pub names: HashSet<String>,
    pub all: bool,
    pub rule: bool,
}

impl AtRootQuery {
    pub fn new(include: bool, names: HashSet<String>) -> Self {
        let all = names.contains("all");
        let rule = names.contains("rule");

        Self {
            include,
            names,
            all,
            rule,
        }
    }

    pub fn excludes_name(&self, name: &str) -> bool {
        (self.all || self.names.contains(name)) != self.include
    }

    pub fn excludes_style_rules(&self) -> bool {
        (self.all || self.rule) != self.include
    }

    pub(crate) fn excludes(&self, stmt: &CssStmt) -> bool {
        if self.all {
            return !self.include;
        }

        match stmt {
            CssStmt::RuleSet { .. } => self.excludes_style_rules(),
            CssStmt::Media(..) => self.excludes_name("media"),
            CssStmt::Supports(..) => self.excludes_name("supports"),
            CssStmt::UnknownAtRule(rule, ..) => self.excludes_name(&rule.name.to_ascii_lowercase()),
            _ => false,
        }
    }
}

impl Default for AtRootQuery {
    fn default() -> Self {
        Self {
            include: false,
            names: HashSet::new(),
            all: false,
            rule: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstImportRule {
    pub imports: Vec<AstImport>,
}

#[derive(Debug, Clone)]
pub enum AstImport {
    Plain(AstPlainCssImport),
    Sass(AstSassImport),
}

impl AstImport {
    pub fn is_dynamic(&self) -> bool {
        matches!(self, AstImport::Sass(..))
    }
}

#[derive(Debug, Clone)]
pub struct AstUseRule {
    pub url: PathBuf,
    pub namespace: Option<String>,
    pub configuration: Vec<ConfiguredVariable>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConfiguredVariable {
    pub name: Spanned<Identifier>,
    pub expr: Spanned<AstExpr>,
    pub is_guarded: bool,
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub(crate) values: Arc<dyn MapView<Value = ConfiguredValue>>,
    #[allow(unused)]
    pub(crate) original_config: Option<Rc<RefCell<Self>>>,
    pub(crate) span: Option<Span>,
}

impl Configuration {
    pub fn through_forward(
        config: Rc<RefCell<Self>>,
        forward: &AstForwardRule,
    ) -> Rc<RefCell<Self>> {
        if (*config).borrow().is_empty() {
            return Rc::new(RefCell::new(Configuration::empty()));
        }

        let mut new_values = Arc::clone(&(*config).borrow().values);

        // Only allow variables that are visible through the `@forward` to be
        // configured. These views support [Map.remove] so we can mark when a
        // configuration variable is used by removing it even when the underlying
        // map is wrapped.
        if let Some(prefix) = &forward.prefix {
            new_values = Arc::new(UnprefixedMapView(new_values, prefix.clone()));
        }

        if let Some(shown_variables) = &forward.shown_variables {
            new_values = Arc::new(LimitedMapView::safelist(new_values, shown_variables));
        } else if let Some(hidden_variables) = &forward.hidden_variables {
            new_values = Arc::new(LimitedMapView::blocklist(new_values, hidden_variables));
        }

        Rc::new(RefCell::new(Self::with_values(
            config,
            Arc::clone(&new_values),
        )))
    }

    fn with_values(
        config: Rc<RefCell<Self>>,
        values: Arc<dyn MapView<Value = ConfiguredValue>>,
    ) -> Self {
        Self {
            values,
            original_config: Some(config),
            span: None,
        }
    }

    pub fn first(&self) -> Option<Spanned<Identifier>> {
        let name = *self.values.keys().first()?;
        let value = self.values.get(name)?;

        Some(Spanned {
            node: name,
            span: value.configuration_span?,
        })
    }

    pub fn remove(&mut self, name: Identifier) -> Option<ConfiguredValue> {
        self.values.remove(name)
    }

    pub fn is_implicit(&self) -> bool {
        self.span.is_none()
    }

    pub fn implicit(values: BTreeMap<Identifier, ConfiguredValue>) -> Self {
        Self {
            values: Arc::new(BaseMapView(Arc::new(RefCell::new(values)))),
            original_config: None,
            span: None,
        }
    }

    pub fn explicit(values: BTreeMap<Identifier, ConfiguredValue>, span: Span) -> Self {
        Self {
            values: Arc::new(BaseMapView(Arc::new(RefCell::new(values)))),
            original_config: None,
            span: Some(span),
        }
    }

    pub fn empty() -> Self {
        Self {
            values: Arc::new(BaseMapView(Arc::new(RefCell::new(BTreeMap::new())))),
            original_config: None,
            span: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[allow(unused)]
    pub fn original_config(config: Rc<RefCell<Configuration>>) -> Rc<RefCell<Configuration>> {
        match (*config).borrow().original_config.as_ref() {
            Some(v) => Rc::clone(v),
            None => Rc::clone(&config),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguredValue {
    pub value: Value,
    pub configuration_span: Option<Span>,
}

impl ConfiguredValue {
    pub fn explicit(value: Value, configuration_span: Span) -> Self {
        Self {
            value,
            configuration_span: Some(configuration_span),
        }
    }

    pub fn implicit(value: Value) -> Self {
        Self {
            value,
            configuration_span: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstForwardRule {
    pub url: PathBuf,
    pub shown_mixins_and_functions: Option<HashSet<Identifier>>,
    pub shown_variables: Option<HashSet<Identifier>>,
    pub hidden_mixins_and_functions: Option<HashSet<Identifier>>,
    pub hidden_variables: Option<HashSet<Identifier>>,
    pub prefix: Option<String>,
    pub configuration: Vec<ConfiguredVariable>,
    pub span: Span,
}

impl AstForwardRule {
    pub fn new(
        url: PathBuf,
        prefix: Option<String>,
        configuration: Option<Vec<ConfiguredVariable>>,
        span: Span,
    ) -> Self {
        Self {
            url,
            shown_mixins_and_functions: None,
            shown_variables: None,
            hidden_mixins_and_functions: None,
            hidden_variables: None,
            prefix,
            configuration: configuration.unwrap_or_default(),
            span,
        }
    }

    pub fn show(
        url: PathBuf,
        shown_mixins_and_functions: HashSet<Identifier>,
        shown_variables: HashSet<Identifier>,
        prefix: Option<String>,
        configuration: Option<Vec<ConfiguredVariable>>,
        span: Span,
    ) -> Self {
        Self {
            url,
            shown_mixins_and_functions: Some(shown_mixins_and_functions),
            shown_variables: Some(shown_variables),
            hidden_mixins_and_functions: None,
            hidden_variables: None,
            prefix,
            configuration: configuration.unwrap_or_default(),
            span,
        }
    }

    pub fn hide(
        url: PathBuf,
        hidden_mixins_and_functions: HashSet<Identifier>,
        hidden_variables: HashSet<Identifier>,
        prefix: Option<String>,
        configuration: Option<Vec<ConfiguredVariable>>,
        span: Span,
    ) -> Self {
        Self {
            url,
            shown_mixins_and_functions: None,
            shown_variables: None,
            hidden_mixins_and_functions: Some(hidden_mixins_and_functions),
            hidden_variables: Some(hidden_variables),
            prefix,
            configuration: configuration.unwrap_or_default(),
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstSupportsCondition {
    Anything {
        contents: Interpolation,
    },
    Declaration {
        name: AstExpr,
        value: AstExpr,
    },
    Function {
        name: Interpolation,
        args: Interpolation,
    },
    Interpolation(AstExpr),
    Negation(Box<Self>),
    Operation {
        left: Box<Self>,
        operator: Option<String>,
        right: Box<Self>,
    },
}

#[derive(Debug, Clone)]
pub struct AstSupportsRule {
    pub condition: AstSupportsCondition,
    pub body: Vec<AstStmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstStmt {
    If(AstIf),
    For(AstFor),
    Return(AstReturn),
    RuleSet(AstRuleSet),
    Style(AstStyle),
    Each(AstEach),
    Media(AstMedia),
    Include(AstInclude),
    While(AstWhile),
    VariableDecl(AstVariableDecl),
    LoudComment(AstLoudComment),
    SilentComment(AstSilentComment),
    FunctionDecl(AstFunctionDecl),
    Mixin(AstMixin),
    ContentRule(AstContentRule),
    Warn(AstWarn),
    UnknownAtRule(AstUnknownAtRule),
    ErrorRule(AstErrorRule),
    Extend(AstExtendRule),
    AtRootRule(AstAtRootRule),
    Debug(AstDebugRule),
    ImportRule(AstImportRule),
    Use(AstUseRule),
    Forward(AstForwardRule),
    Supports(AstSupportsRule),
}

#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub body: Vec<AstStmt>,
    pub url: PathBuf,
    pub is_plain_css: bool,
    /// Array of indices into `body`
    pub uses: Vec<usize>,
    /// Array of indices into `body`
    pub forwards: Vec<usize>,
}

impl StyleSheet {
    pub fn new(is_plain_css: bool, url: PathBuf) -> Self {
        Self {
            body: Vec::new(),
            url,
            is_plain_css,
            uses: Vec::new(),
            forwards: Vec::new(),
        }
    }
}
