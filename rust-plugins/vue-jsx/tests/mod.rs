use std::sync::Arc;

use farmfe_core::{
    config::Config,
    context::CompilationContext,
    module::ModuleType,
    plugin::{Plugin, PluginTransformHookParam},
};
use farmfe_plugin_vue_jsx::FarmPluginVueJsx;

fn make_plugin(options: &str) -> FarmPluginVueJsx {
    let config = Config::default();
    FarmPluginVueJsx::new(&config, options.to_string())
}

fn transform_jsx(input: &str, filename: &str, module_type: ModuleType) -> String {
    let plugin = make_plugin("{}");
    let config = Config::default();
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let transform_param = PluginTransformHookParam {
        module_id: filename.to_string(),
        content: input.to_string(),
        module_type,
        resolved_path: filename,
        query: vec![],
        meta: Default::default(),
        source_map_chain: vec![],
    };
    plugin
        .transform(&transform_param, &context)
        .expect("transform returns Ok")
        .expect("transform returns Some for jsx/tsx")
        .content
}

fn assert_transform(fixture_name: &str) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture_name);
    let input = std::fs::read_to_string(dir.join("input.jsx")).unwrap();
    let expected = std::fs::read_to_string(dir.join("output.js")).unwrap();
    let output = transform_jsx(
        &input,
        &format!("{fixture_name}.tsx"),
        ModuleType::Tsx,
    );
    assert_eq!(
        output.trim(),
        expected.trim(),
        "mismatch for fixture: {fixture_name}"
    );
}

#[test]
fn ignores_non_jsx_files() {
    let plugin = make_plugin("{}");
    let config = Config::default();
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let param = PluginTransformHookParam {
        module_id: "test.js".to_string(),
        content: "const x = 1;".to_string(),
        module_type: ModuleType::Js,
        resolved_path: "test.js",
        query: vec![],
        meta: Default::default(),
        source_map_chain: vec![],
    };
    assert!(plugin.transform(&param, &context).unwrap().is_none());
}

#[test] fn custom_directive() { assert_transform("custom-directive"); }
#[test] fn custom_directive_with_argument_and_modifiers() { assert_transform("custom-directive-with-argument-and-modifiers"); }
#[test] fn custom_element() { assert_transform("custom-element"); }
#[test] fn custom_pragma_in_comment() { assert_transform("custom-pragma-in-comment"); }
#[test] fn custom_pragma_in_options() { assert_transform("custom-pragma-in-options"); }
#[test] fn disable_object_slot() { assert_transform("disable-object-slot"); }
#[test] fn empty_string() { assert_transform("empty-string"); }
#[test] fn fragment_already_imported() { assert_transform("fragment-already-imported"); }
#[test] fn function_expr_slot() { assert_transform("function-expr-slot"); }
#[test] fn keep_alive_named_import() { assert_transform("keep-alive-named-import"); }
#[test] fn keep_alive_namespace_import() { assert_transform("keep-alive-namespace-import"); }
#[test] fn keep_namespace_import() { assert_transform("keep-namespace-import"); }
#[test] fn merge_class_style_attrs() { assert_transform("merge-class-style-attrs"); }
#[test] fn merge_props_order() { assert_transform("merge-props-order"); }
#[test] fn model_as_prop_name() { assert_transform("model-as-prop-name"); }
#[test] fn multiple_exprs_slot() { assert_transform("multiple-exprs-slot"); }
#[test] fn nesting_slot_flags() { assert_transform("nesting-slot-flags"); }
#[test] fn non_literal_expr_slot() { assert_transform("non-literal-expr-slot"); }
#[test] fn override_props_multiple() { assert_transform("override-props-multiple"); }
#[test] fn override_props_single() { assert_transform("override-props-single"); }
#[test] fn reassign_variable_as_component() { assert_transform("reassign-variable-as-component"); }
#[test] fn single_attr() { assert_transform("single-attr"); }
#[test] fn slot_in_arrow_function_bug() { assert_transform("slot-in-arrow-function-bug"); }
#[test] fn specifiers_merged_into_single_import_decl() { assert_transform("specifiers-merged-into-single-import-decl"); }
#[test] fn v_html() { assert_transform("v-html"); }
#[test] fn v_models() { assert_transform("v-models"); }
#[test] fn v_model_value_supports_variable() { assert_transform("v-model-value-supports-variable"); }
#[test] fn v_model_with_arg_and_modifier() { assert_transform("v-model-with-arg-and-modifier"); }
#[test] fn v_model_with_checkbox() { assert_transform("v-model-with-checkbox"); }
#[test] fn v_model_with_dynamic_type_input() { assert_transform("v-model-with-dynamic-type-input"); }
#[test] fn v_model_with_input_lazy_modifier() { assert_transform("v-model-with-input-lazy-modifier"); }
#[test] fn v_model_with_radio() { assert_transform("v-model-with-radio"); }
#[test] fn v_model_with_select() { assert_transform("v-model-with-select"); }
#[test] fn v_model_with_textarea() { assert_transform("v-model-with-textarea"); }
#[test] fn v_model_with_text_input() { assert_transform("v-model-with-text-input"); }
#[test] fn v_show() { assert_transform("v-show"); }
#[test] fn v_slots() { assert_transform("v-slots"); }
#[test] fn v_slots_complex() { assert_transform("v-slots-complex"); }
#[test] fn v_text() { assert_transform("v-text"); }
#[test] fn without_jsx() { assert_transform("without-jsx"); }
#[test] fn without_props() { assert_transform("without-props"); }
