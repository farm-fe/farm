use codemap::{Span, Spanned};

use crate::{
    ast::{AstForwardRule, Configuration, ConfiguredValue, Mixin},
    builtin::modules::{ForwardedModule, Module, ModuleScope, Modules, ShadowedModule},
    common::Identifier,
    error::SassResult,
    selector::ExtensionStore,
    value::{SassFunction, Value},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

type Mutable<T> = Arc<RefCell<T>>;

use super::{scope::Scopes, visitor::CallableContentBlock};

#[derive(Debug, Clone)]
pub(crate) struct Environment {
    pub scopes: Scopes,
    pub modules: Mutable<Modules>,
    pub global_modules: Vec<Mutable<Module>>,
    pub content: Option<Arc<CallableContentBlock>>,
    pub forwarded_modules: Mutable<Vec<Mutable<Module>>>,
    pub imported_modules: Mutable<Vec<Mutable<Module>>>,
    #[allow(clippy::type_complexity)]
    pub nested_forwarded_modules: Option<Mutable<Vec<Mutable<Vec<Mutable<Module>>>>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: Scopes::new(),
            modules: Arc::new(RefCell::new(Modules::new())),
            global_modules: Vec::new(),
            content: None,
            forwarded_modules: Arc::new(RefCell::new(Vec::new())),
            imported_modules: Arc::new(RefCell::new(Vec::new())),
            nested_forwarded_modules: None,
        }
    }

    pub fn new_closure(&self) -> Self {
        Self {
            scopes: self.scopes.new_closure(),
            modules: Arc::clone(&self.modules),
            global_modules: self.global_modules.iter().map(Arc::clone).collect(),
            content: self.content.as_ref().map(Arc::clone),
            forwarded_modules: Arc::clone(&self.forwarded_modules),
            imported_modules: Arc::clone(&self.imported_modules),
            nested_forwarded_modules: self.nested_forwarded_modules.as_ref().map(Arc::clone),
        }
    }

    pub fn for_import(&self) -> Self {
        Self {
            scopes: self.scopes.new_closure(),
            modules: Arc::new(RefCell::new(Modules::new())),
            global_modules: Vec::new(),
            content: self.content.as_ref().map(Arc::clone),
            forwarded_modules: Arc::clone(&self.forwarded_modules),
            imported_modules: Arc::clone(&self.imported_modules),
            nested_forwarded_modules: self.nested_forwarded_modules.as_ref().map(Arc::clone),
        }
    }

    pub fn to_dummy_module(&self, span: Span) -> Module {
        Module::Environment {
            scope: ModuleScope::new(),
            upstream: Vec::new(),
            extension_store: ExtensionStore::new(span),
            env: self.clone(),
        }
    }

    /// Makes the members forwarded by [module] available in the current
    /// environment.
    ///
    /// This is called when [module] is `@import`ed.
    pub fn import_forwards(&mut self, _env: Module) {
        if let Module::Environment { env, .. } = _env {
            let mut forwarded = env.forwarded_modules;

            if (*forwarded).borrow().is_empty() {
                return;
            }

            // Omit modules from [forwarded] that are already globally available and
            // forwarded in this module.
            let forwarded_modules = Arc::clone(&self.forwarded_modules);
            if !(*forwarded_modules).borrow().is_empty() {
                // todo: intermediate name
                let mut x = Vec::new();
                for entry in (*forwarded).borrow().iter() {
                    if !forwarded_modules
                        .borrow()
                        .iter()
                        .any(|module| Arc::ptr_eq(module, entry))
                        || !self
                            .global_modules
                            .iter()
                            .any(|module| Arc::ptr_eq(module, entry))
                    {
                        x.push(Arc::clone(entry));
                    }
                }

                forwarded = Arc::new(RefCell::new(x));
            }

            let forwarded_var_names = forwarded
                .borrow()
                .iter()
                .flat_map(|module| (*module).borrow().scope().variables.keys())
                .collect::<HashSet<Identifier>>();
            let forwarded_fn_names = forwarded
                .borrow()
                .iter()
                .flat_map(|module| (*module).borrow().scope().functions.keys())
                .collect::<HashSet<Identifier>>();
            let forwarded_mixin_names = forwarded
                .borrow()
                .iter()
                .flat_map(|module| (*module).borrow().scope().mixins.keys())
                .collect::<HashSet<Identifier>>();

            if self.at_root() {
                let mut to_remove = Vec::new();

                // Hide members from modules that have already been imported or
                // forwarded that would otherwise conflict with the @imported members.
                for (idx, module) in (*self.imported_modules).borrow().iter().enumerate() {
                    let shadowed = ShadowedModule::if_necessary(
                        Arc::clone(module),
                        Some(&forwarded_var_names),
                        Some(&forwarded_fn_names),
                        Some(&forwarded_mixin_names),
                    );

                    if shadowed.is_some() {
                        to_remove.push(idx);
                    }
                }

                let mut imported_modules = (*self.imported_modules).borrow_mut();

                for &idx in to_remove.iter().rev() {
                    imported_modules.remove(idx);
                }

                to_remove.clear();

                for (idx, module) in (*self.forwarded_modules).borrow().iter().enumerate() {
                    let shadowed = ShadowedModule::if_necessary(
                        Arc::clone(module),
                        Some(&forwarded_var_names),
                        Some(&forwarded_fn_names),
                        Some(&forwarded_mixin_names),
                    );

                    if shadowed.is_some() {
                        to_remove.push(idx);
                    }
                }

                let mut forwarded_modules = (*self.forwarded_modules).borrow_mut();

                for &idx in to_remove.iter().rev() {
                    forwarded_modules.remove(idx);
                }

                imported_modules.extend(forwarded.borrow().iter().map(Arc::clone));
                forwarded_modules.extend(forwarded.borrow().iter().map(Arc::clone));
            } else {
                self.nested_forwarded_modules
                    .get_or_insert_with(|| {
                        Arc::new(RefCell::new(
                            (0..self.scopes.len())
                                .map(|_| Arc::new(RefCell::new(Vec::new())))
                                .collect(),
                        ))
                    })
                    .borrow_mut()
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .extend(forwarded.borrow().iter().map(Arc::clone));
            }

            // Remove existing member definitions that are now shadowed by the
            // forwarded modules.
            for variable in forwarded_var_names {
                (*self.scopes.variables)
                    .borrow_mut()
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .remove(&variable);
            }
            self.scopes.last_variable_index = None;

            for func in forwarded_fn_names {
                (*self.scopes.functions)
                    .borrow_mut()
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .remove(&func);
            }
            for mixin in forwarded_mixin_names {
                (*self.scopes.mixins)
                    .borrow_mut()
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .remove(&mixin);
            }
        }
    }

    pub fn to_implicit_configuration(&self) -> Configuration {
        let mut configuration = BTreeMap::new();

        let variables = (*self.scopes.variables).borrow();

        for variables in variables.iter() {
            let entries = (**variables).borrow();
            for (key, value) in entries.iter() {
                // Implicit configurations are never invalid, making [configurationSpan]
                // unnecessary, so we pass null here to avoid having to compute it.
                configuration.insert(*key, ConfiguredValue::implicit(value.clone()));
            }
        }

        Configuration::implicit(configuration)
    }

    pub fn forward_module(&mut self, module: Arc<RefCell<Module>>, rule: AstForwardRule) {
        let view = ForwardedModule::if_necessary(module, rule);
        (*self.forwarded_modules).borrow_mut().push(view);

        // todo: assertnoconflicts
    }

    pub fn insert_mixin(&mut self, name: Identifier, mixin: Mixin) {
        self.scopes.insert_mixin(name, mixin);
    }

    pub fn mixin_exists(&self, name: Identifier) -> bool {
        self.scopes.mixin_exists(name)
    }

    pub fn get_mixin(
        &self,
        name: Spanned<Identifier>,
        namespace: Option<Spanned<Identifier>>,
    ) -> SassResult<Mixin> {
        if let Some(namespace) = namespace {
            let modules = (*self.modules).borrow();
            let module = modules.get(namespace.node, namespace.span)?;
            return (*module).borrow().get_mixin(name);
        }

        match self.scopes.get_mixin(name) {
            Ok(v) => Ok(v),
            Err(e) => {
                if let Some(v) = self.get_mixin_from_global_modules(name.node) {
                    return Ok(v);
                }

                Err(e)
            }
        }
    }

    pub fn insert_fn(&mut self, func: SassFunction) {
        self.scopes.insert_fn(func);
    }

    pub fn fn_exists(&self, name: Identifier) -> bool {
        self.scopes.fn_exists(name)
    }

    pub fn get_fn(
        &self,
        name: Identifier,
        namespace: Option<Spanned<Identifier>>,
    ) -> SassResult<Option<SassFunction>> {
        if let Some(namespace) = namespace {
            let modules = (*self.modules).borrow();
            let module = modules.get(namespace.node, namespace.span)?;
            return Ok((*module).borrow().get_fn(name));
        }

        Ok(self
            .scopes
            .get_fn(name)
            .or_else(|| self.get_function_from_global_modules(name)))
    }

    pub fn var_exists(
        &self,
        name: Identifier,
        namespace: Option<Spanned<Identifier>>,
    ) -> SassResult<bool> {
        if let Some(namespace) = namespace {
            let modules = (*self.modules).borrow();
            let module = modules.get(namespace.node, namespace.span)?;
            return Ok((*module).borrow().var_exists(name));
        }

        Ok(self.scopes.var_exists(name))
    }

    pub fn get_var(
        &mut self,
        name: Spanned<Identifier>,
        namespace: Option<Spanned<Identifier>>,
    ) -> SassResult<Value> {
        if let Some(namespace) = namespace {
            let modules = (*self.modules).borrow();
            let module = modules.get(namespace.node, namespace.span)?;
            return (*module).borrow().get_var(name);
        }

        match self.scopes.get_var(name) {
            Ok(v) => Ok(v),
            Err(e) => {
                if let Some(v) = self.get_variable_from_global_modules(name.node) {
                    Ok(v)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn insert_var(
        &mut self,
        name: Spanned<Identifier>,
        namespace: Option<Spanned<Identifier>>,
        value: Value,
        is_global: bool,
        in_semi_global_scope: bool,
    ) -> SassResult<()> {
        if let Some(namespace) = namespace {
            let mut modules = (*self.modules).borrow_mut();
            let module = modules.get_mut(namespace.node, namespace.span)?;
            (*module).borrow_mut().update_var(name, value)?;
            return Ok(());
        }

        if is_global || self.at_root() {
            // If this module doesn't already contain a variable named [name], try
            // setting it in a global module.
            if !self.scopes.global_var_exists(name.node) {
                let module_with_name = self.from_one_module(name.node, "variable", |module| {
                    if module.borrow().var_exists(*name) {
                        Some(Arc::clone(module))
                    } else {
                        None
                    }
                });

                if let Some(module_with_name) = module_with_name {
                    module_with_name.borrow_mut().update_var(name, value)?;
                    return Ok(());
                }
            }

            self.scopes.insert_var(0, name.node, value);
            return Ok(());
        }

        let mut index = self
            .scopes
            .find_var(name.node)
            .unwrap_or(self.scopes.len() - 1);

        if !in_semi_global_scope && index == 0 {
            index = self.scopes.len() - 1;
        }

        self.scopes.last_variable_index = Some((name.node, index));

        self.scopes.insert_var(index, name.node, value);

        Ok(())
    }

    pub fn at_root(&self) -> bool {
        self.scopes.len() == 1
    }

    pub fn scopes_mut(&mut self) -> &mut Scopes {
        &mut self.scopes
    }

    pub fn global_vars(&self) -> Arc<RefCell<BTreeMap<Identifier, Value>>> {
        self.scopes.global_variables()
    }

    pub fn global_mixins(&self) -> Arc<RefCell<BTreeMap<Identifier, Mixin>>> {
        self.scopes.global_mixins()
    }

    pub fn global_functions(&self) -> Arc<RefCell<BTreeMap<Identifier, SassFunction>>> {
        self.scopes.global_functions()
    }

    fn get_variable_from_global_modules(&self, name: Identifier) -> Option<Value> {
        self.from_one_module(name, "variable", |module| {
            (**module).borrow().get_var_no_err(name)
        })
    }

    fn get_function_from_global_modules(&self, name: Identifier) -> Option<SassFunction> {
        self.from_one_module(name, "function", |module| (**module).borrow().get_fn(name))
    }

    fn get_mixin_from_global_modules(&self, name: Identifier) -> Option<Mixin> {
        self.from_one_module(name, "mixin", |module| {
            (**module).borrow().get_mixin_no_err(name)
        })
    }

    pub fn add_module(
        &mut self,
        namespace: Option<Identifier>,
        module: Arc<RefCell<Module>>,
        span: Span,
    ) -> SassResult<()> {
        match namespace {
            Some(namespace) => {
                (*self.modules)
                    .borrow_mut()
                    .insert(namespace, module, span)?;
            }
            None => {
                for name in (*self.scopes.global_variables()).borrow().keys() {
                    if (*module).borrow().var_exists(*name) {
                        return Err((
                            format!("This module and the new module both define a variable named \"${name}\".", name = name)
                        , span).into());
                    }
                }

                self.global_modules.push(module);
            }
        }

        Ok(())
    }

    pub fn to_module(self, extension_store: ExtensionStore) -> Arc<RefCell<Module>> {
        debug_assert!(self.at_root());

        Arc::new(RefCell::new(Module::new_env(self, extension_store)))
    }

    fn from_one_module<T>(
        &self,
        _name: Identifier,
        _ty: &str,
        callback: impl Fn(&Arc<RefCell<Module>>) -> Option<T>,
    ) -> Option<T> {
        if let Some(nested_forwarded_modules) = &self.nested_forwarded_modules {
            for modules in nested_forwarded_modules.borrow().iter().rev() {
                for module in modules.borrow().iter().rev() {
                    if let Some(value) = callback(module) {
                        return Some(value);
                    }
                }
            }
        }

        for module in self.imported_modules.borrow().iter() {
            if let Some(value) = callback(module) {
                return Some(value);
            }
        }

        let mut value: Option<T> = None;
        //     Object? identity;

        for module in self.global_modules.iter() {
            let value_in_module = match callback(module) {
                Some(v) => v,
                None => continue,
            };

            value = Some(value_in_module);

            //       Object? identityFromModule = valueInModule is AsyncCallable
            //           ? valueInModule
            //           : module.variableIdentity(name);
            //       if (identityFromModule == identity) continue;

            //       if (value != null) {
            //         var spans = _globalModules.entries.map(
            //             (entry) => callback(entry.key).andThen((_) => entry.value.span));

            //         throw MultiSpanSassScriptException(
            //             'This $type is available from multiple global modules.',
            //             '$type use', {
            //           for (var span in spans)
            //             if (span != null) span: 'includes $type'
            //         });
            //       }

            //       value = valueInModule;
            //       identity = identityFromModule;
        }

        value
    }
}
