use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    fmt,
    sync::Arc,
};

use codemap::{Span, Spanned};

use crate::{
    ast::{ArgumentResult, AstForwardRule, BuiltinMixin, Mixin},
    builtin::Builtin,
    common::Identifier,
    error::SassResult,
    evaluate::{Environment, Visitor},
    selector::ExtensionStore,
    utils::{
        BaseMapView, LimitedMapView, MapView, MergedMapView, PrefixedMapView, PublicMemberMapView,
    },
    value::{SassFunction, SassMap, Value},
};

use super::builtin_imports::QuoteKind;

mod color;
mod list;
mod map;
mod math;
mod meta;
mod selector;
mod string;

/// A [Module] that only exposes members that aren't shadowed by a given
/// blocklist of member names.
#[derive(Debug, Clone)]
pub(crate) struct ShadowedModule {
    #[allow(dead_code)]
    inner: Arc<RefCell<Module>>,
    scope: ModuleScope,
}

impl ShadowedModule {
    pub fn new(
        module: Arc<RefCell<Module>>,
        variables: Option<&HashSet<Identifier>>,
        functions: Option<&HashSet<Identifier>>,
        mixins: Option<&HashSet<Identifier>>,
    ) -> Self {
        let module_scope = module.borrow().scope();

        let variables = Self::shadowed_map(Arc::clone(&module_scope.variables), variables);
        let functions = Self::shadowed_map(Arc::clone(&module_scope.functions), functions);
        let mixins = Self::shadowed_map(Arc::clone(&module_scope.mixins), mixins);

        let new_scope = ModuleScope {
            variables,
            functions,
            mixins,
        };

        Self {
            inner: module,
            scope: new_scope,
        }
    }

    fn needs_blocklist<V: fmt::Debug + Clone>(
        map: Arc<dyn MapView<Value = V>>,
        blocklist: Option<&HashSet<Identifier>>,
    ) -> bool {
        blocklist.is_some()
            && !map.is_empty()
            && blocklist.unwrap().iter().any(|key| map.contains_key(*key))
    }

    fn shadowed_map<V: fmt::Debug + Clone + 'static>(
        map: Arc<dyn MapView<Value = V>>,
        blocklist: Option<&HashSet<Identifier>>,
    ) -> Arc<dyn MapView<Value = V>> {
        match blocklist {
            Some(..) if !Self::needs_blocklist(Arc::clone(&map), blocklist) => map,
            Some(blocklist) => Arc::new(LimitedMapView::blocklist(map, blocklist)),
            None => map,
        }
    }

    pub fn if_necessary(
        module: Arc<RefCell<Module>>,
        variables: Option<&HashSet<Identifier>>,
        functions: Option<&HashSet<Identifier>>,
        mixins: Option<&HashSet<Identifier>>,
    ) -> Option<Arc<RefCell<Module>>> {
        let module_scope = module.borrow().scope();

        let needs_blocklist = Self::needs_blocklist(Arc::clone(&module_scope.variables), variables)
            || Self::needs_blocklist(Arc::clone(&module_scope.functions), functions)
            || Self::needs_blocklist(Arc::clone(&module_scope.mixins), mixins);

        if needs_blocklist {
            Some(Arc::new(RefCell::new(Module::Shadowed(Self::new(
                module, variables, functions, mixins,
            )))))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ForwardedModule {
    scope: ModuleScope,
    #[allow(dead_code)]
    inner: Arc<RefCell<Module>>,
    #[allow(dead_code)]
    forward_rule: AstForwardRule,
}

impl ForwardedModule {
    pub fn new(module: Arc<RefCell<Module>>, rule: AstForwardRule) -> Self {
        let scope = (*module).borrow().scope();

        let variables = Self::forwarded_map(
            scope.variables,
            rule.prefix.as_deref(),
            rule.shown_variables.as_ref(),
            rule.hidden_variables.as_ref(),
        );

        let functions = Self::forwarded_map(
            scope.functions,
            rule.prefix.as_deref(),
            rule.shown_mixins_and_functions.as_ref(),
            rule.hidden_mixins_and_functions.as_ref(),
        );

        let mixins = Self::forwarded_map(
            scope.mixins,
            rule.prefix.as_deref(),
            rule.shown_mixins_and_functions.as_ref(),
            rule.hidden_mixins_and_functions.as_ref(),
        );

        let scope = ModuleScope {
            variables,
            mixins,
            functions,
        };

        ForwardedModule {
            inner: module,
            forward_rule: rule,
            scope,
        }
    }

    fn forwarded_map<T: Clone + fmt::Debug + 'static>(
        mut map: Arc<dyn MapView<Value = T>>,
        prefix: Option<&str>,
        safelist: Option<&HashSet<Identifier>>,
        blocklist: Option<&HashSet<Identifier>>,
    ) -> Arc<dyn MapView<Value = T>> {
        debug_assert!(safelist.is_none() || blocklist.is_none());

        if prefix.is_none() && safelist.is_none() && blocklist.is_none() {
            return map;
        }

        if let Some(prefix) = prefix {
            map = Arc::new(PrefixedMapView(map, prefix.to_owned()));
        }

        map
    }

    pub fn if_necessary(
        module: Arc<RefCell<Module>>,
        rule: AstForwardRule,
    ) -> Arc<RefCell<Module>> {
        if rule.prefix.is_none()
            && rule.shown_mixins_and_functions.is_none()
            && rule.shown_variables.is_none()
            && rule
                .hidden_mixins_and_functions
                .as_ref()
                .map_or(false, HashSet::is_empty)
            && rule
                .hidden_variables
                .as_ref()
                .map_or(false, HashSet::is_empty)
        {
            module
        } else {
            Arc::new(RefCell::new(Module::Forwarded(ForwardedModule::new(
                module, rule,
            ))))
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ModuleScope {
    pub variables: Arc<dyn MapView<Value = Value>>,
    pub mixins: Arc<dyn MapView<Value = Mixin>>,
    pub functions: Arc<dyn MapView<Value = SassFunction>>,
}

impl ModuleScope {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(BaseMapView(Arc::new(RefCell::new(BTreeMap::new())))),
            mixins: Arc::new(BaseMapView(Arc::new(RefCell::new(BTreeMap::new())))),
            functions: Arc::new(BaseMapView(Arc::new(RefCell::new(BTreeMap::new())))),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Module {
    Environment {
        scope: ModuleScope,
        #[allow(dead_code)]
        upstream: Vec<Module>,
        #[allow(dead_code)]
        extension_store: ExtensionStore,
        #[allow(dead_code)]
        env: Environment,
    },
    Builtin {
        scope: ModuleScope,
    },
    Forwarded(ForwardedModule),
    Shadowed(ShadowedModule),
}

#[derive(Debug, Clone)]
pub(crate) struct Modules(pub BTreeMap<Identifier, Arc<RefCell<Module>>>);

impl Modules {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(
        &mut self,
        name: Identifier,
        module: Arc<RefCell<Module>>,
        span: Span,
    ) -> SassResult<()> {
        if self.0.contains_key(&name) {
            return Err((
                format!("There's already a module with namespace \"{}\".", name),
                span,
            )
                .into());
        }

        self.0.insert(name, module);

        Ok(())
    }

    pub fn get(&self, name: Identifier, span: Span) -> SassResult<Arc<RefCell<Module>>> {
        match self.0.get(&name) {
            Some(v) => Ok(Arc::clone(v)),
            None => Err((
                format!(
                    "There is no module with the namespace \"{}\".",
                    name.as_str()
                ),
                span,
            )
                .into()),
        }
    }

    pub fn get_mut(
        &mut self,
        name: Identifier,
        span: Span,
    ) -> SassResult<&mut Arc<RefCell<Module>>> {
        match self.0.get_mut(&name) {
            Some(v) => Ok(v),
            None => Err((
                format!(
                    "There is no module with the namespace \"{}\".",
                    name.as_str()
                ),
                span,
            )
                .into()),
        }
    }
}

fn member_map<V: fmt::Debug + Clone + 'static>(
    local: Arc<dyn MapView<Value = V>>,
    others: Vec<Arc<dyn MapView<Value = V>>>,
) -> Arc<dyn MapView<Value = V>> {
    let local_map = PublicMemberMapView(local);

    if others.is_empty() {
        return Arc::new(local_map);
    }

    let mut all_maps: Vec<Arc<dyn MapView<Value = V>>> =
        others.into_iter().filter(|map| !map.is_empty()).collect();

    all_maps.push(Arc::new(local_map));

    // todo: potential optimization when all_maps.len() == 1
    Arc::new(MergedMapView::new(all_maps))
}

impl Module {
    pub fn new_env(env: Environment, extension_store: ExtensionStore) -> Self {
        let variables = {
            let variables = (*env.forwarded_modules).borrow();
            let variables = variables
                .iter()
                .map(|module| Arc::clone(&(*module).borrow().scope().variables));
            let this = Arc::new(BaseMapView(env.global_vars()));
            member_map(this, variables.collect())
        };

        let mixins = {
            let mixins = (*env.forwarded_modules).borrow();
            let mixins = mixins
                .iter()
                .map(|module| Arc::clone(&(*module).borrow().scope().mixins));
            let this = Arc::new(BaseMapView(env.global_mixins()));
            member_map(this, mixins.collect())
        };

        let functions = {
            let functions = (*env.forwarded_modules).borrow();
            let functions = functions
                .iter()
                .map(|module| Arc::clone(&(*module).borrow().scope().functions));
            let this = Arc::new(BaseMapView(env.global_functions()));
            member_map(this, functions.collect())
        };

        let scope = ModuleScope {
            variables,
            mixins,
            functions,
        };

        Module::Environment {
            scope,
            upstream: Vec::new(),
            extension_store,
            env,
        }
    }

    pub fn new_builtin() -> Self {
        Module::Builtin {
            scope: ModuleScope::new(),
        }
    }

    pub(crate) fn scope(&self) -> ModuleScope {
        match self {
            Self::Builtin { scope }
            | Self::Environment { scope, .. }
            | Self::Forwarded(ForwardedModule { scope, .. })
            | Self::Shadowed(ShadowedModule { scope, .. }) => scope.clone(),
        }
    }

    pub fn get_var(&self, name: Spanned<Identifier>) -> SassResult<Value> {
        let scope = self.scope();

        match scope.variables.get(name.node) {
            Some(v) => Ok(v),
            None => Err(("Undefined variable.", name.span).into()),
        }
    }

    pub fn get_var_no_err(&self, name: Identifier) -> Option<Value> {
        let scope = self.scope();

        scope.variables.get(name)
    }

    pub fn get_mixin_no_err(&self, name: Identifier) -> Option<Mixin> {
        let scope = self.scope();

        scope.mixins.get(name)
    }

    pub fn update_var(&mut self, name: Spanned<Identifier>, value: Value) -> SassResult<()> {
        let scope = match self {
            Self::Builtin { .. } => {
                return Err(("Cannot modify built-in variable.", name.span).into())
            }
            Self::Environment { scope, .. }
            | Self::Forwarded(ForwardedModule { scope, .. })
            | Self::Shadowed(ShadowedModule { scope, .. }) => scope.clone(),
        };

        if scope.variables.insert(name.node, value).is_none() {
            return Err(("Undefined variable.", name.span).into());
        }

        Ok(())
    }

    pub fn get_mixin(&self, name: Spanned<Identifier>) -> SassResult<Mixin> {
        let scope = self.scope();

        match scope.mixins.get(name.node) {
            Some(v) => Ok(v),
            None => Err(("Undefined mixin.", name.span).into()),
        }
    }

    pub fn insert_builtin_mixin(&mut self, name: &'static str, mixin: BuiltinMixin) {
        let scope = self.scope();

        scope.mixins.insert(name.into(), Mixin::Builtin(mixin));
    }

    pub fn insert_builtin_var(&mut self, name: &'static str, value: Value) {
        let ident = name.into();

        let scope = self.scope();

        scope.variables.insert(ident, value);
    }

    pub fn get_fn(&self, name: Identifier) -> Option<SassFunction> {
        let scope = self.scope();

        scope.functions.get(name)
    }

    pub fn var_exists(&self, name: Identifier) -> bool {
        let scope = self.scope();

        scope.variables.get(name).is_some()
    }

    pub fn mixin_exists(&self, name: Identifier) -> bool {
        let scope = self.scope();

        scope.mixins.get(name).is_some()
    }

    pub fn fn_exists(&self, name: Identifier) -> bool {
        let scope = self.scope();

        scope.functions.get(name).is_some()
    }

    pub fn insert_builtin(
        &mut self,
        name: &'static str,
        function: fn(ArgumentResult, &mut Visitor) -> SassResult<Value>,
    ) {
        let ident = name.into();

        let scope = match self {
            Self::Builtin { scope } => scope,
            _ => unreachable!(),
        };

        scope
            .functions
            .insert(ident, SassFunction::Builtin(Builtin::new(function), ident));
    }

    pub fn functions(&self, span: Span) -> SassMap {
        SassMap::new_with(
            self.scope()
                .functions
                .iter()
                .into_iter()
                .filter(|(key, _)| !key.as_str().starts_with('-'))
                .map(|(key, value)| {
                    (
                        Value::String(key.to_string(), QuoteKind::Quoted).span(span),
                        Value::FunctionRef(Box::new(value)),
                    )
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn variables(&self, span: Span) -> SassMap {
        SassMap::new_with(
            self.scope()
                .variables
                .iter()
                .into_iter()
                .filter(|(key, _)| !key.as_str().starts_with('-'))
                .map(|(key, value)| {
                    (
                        Value::String(key.to_string(), QuoteKind::Quoted).span(span),
                        value,
                    )
                })
                .collect::<Vec<_>>(),
        )
    }
}

pub(crate) fn declare_module_color() -> Module {
    let mut module = Module::new_builtin();
    color::declare(&mut module);
    module
}

pub(crate) fn declare_module_list() -> Module {
    let mut module = Module::new_builtin();
    list::declare(&mut module);
    module
}

pub(crate) fn declare_module_map() -> Module {
    let mut module = Module::new_builtin();
    map::declare(&mut module);
    module
}

pub(crate) fn declare_module_math() -> Module {
    let mut module = Module::new_builtin();
    math::declare(&mut module);
    module
}

pub(crate) fn declare_module_meta() -> Module {
    let mut module = Module::new_builtin();
    meta::declare(&mut module);
    module
}

pub(crate) fn declare_module_selector() -> Module {
    let mut module = Module::new_builtin();
    selector::declare(&mut module);
    module
}

pub(crate) fn declare_module_string() -> Module {
    let mut module = Module::new_builtin();
    string::declare(&mut module);
    module
}
