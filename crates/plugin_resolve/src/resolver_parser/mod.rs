pub fn resolve_strategy(
  source: &str,
  base_dir: PathBuf,
  kind: &ResolveKind,
  context: &Arc<CompilationContext>,
  package_json_info: Option<PackageJsonInfo>,
) -> Option<PluginResolveHookResult> {
  // 在这里根据 source 和其他参数选择合适的解析策略
  if let Some(result) = try_alias(source, base_dir.clone(), kind, context) {
    Some(result)
  } else if is_source_absolute(source) {
    let path_buf = PathBuf::from_str(source).unwrap();
    // 使用 AbsoluteStrategy 解析策略
    AbsoluteStrategy.resolve(source, base_dir, kind, context, package_json_info)
  } else if is_source_relative(source) {
    farm_profile_scope!("resolve.relative".to_string());
    // 使用 RelativeStrategy 解析策略
    RelativeStrategy.resolve(source, base_dir, kind, context, package_json_info)
  } else if is_source_dot(source) {
    // 使用 DotStrategy 解析策略
    DotStrategy.resolve(source, base_dir, kind, context, package_json_info)
  } else if is_source_double_dot(source) {
    // 使用 DoubleDotStrategy 解析策略
    DoubleDotStrategy.resolve(source, base_dir, kind, context, package_json_info)
  } else {
    // 使用默认解析策略
    None
  }
}
