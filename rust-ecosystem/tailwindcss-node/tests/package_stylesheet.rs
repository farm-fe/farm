use farmfe_ecosystem_tailwindcss_node::compile::{compile, CompileOptions};
use std::{
  fs,
  sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
#[ignore = "requires an independently installed Tailwind package; pass FARM_TAILWIND_PACKAGE_BASE"]
fn installed_tailwind_package_preserves_real_base_styles() {
  let base = std::env::var("FARM_TAILWIND_PACKAGE_BASE").expect("independent consumer base");
  let mut compiler = compile(
    "@import 'tailwindcss';",
    CompileOptions {
      base,
      ..Default::default()
    },
  )
  .expect("compile actual installed package");
  let css = compiler.build(&["grid".to_string(), "md:grid-cols-2".to_string()]);
  assert!(css.contains("@layer base"), "missing installed preflight");
  assert!(
    css.contains("@layer theme {\n  :root, :host {"),
    "theme variables escaped their cascade layer"
  );
  assert!(
    css.contains("border: 0 solid"),
    "missing installed border reset"
  );
  assert!(
    css.contains("--default-font-family"),
    "missing installed theme reference"
  );
  assert!(
    css.contains(".md\\:grid-cols-2"),
    "missing responsive class identity"
  );
  assert!(!css.contains("@theme"), "uncompiled theme directive");
  assert!(!css.contains("@tailwind"), "uncompiled utilities directive");
  assert!(
    !css.contains("--theme("),
    "uncompiled internal theme function"
  );
  assert!(compiler
    .dependencies()
    .iter()
    .any(|path| path.ends_with("index.css")));
  if let Ok(output) = std::env::var("FARM_TAILWIND_PACKAGE_OUTPUT") {
    fs::write(output, css).expect("write actual package output evidence");
  }
}

#[test]
fn tailwind_package_import_preserves_preflight_and_tracks_dependency() {
  let root = std::env::temp_dir().join(format!(
    "farm-tailwind-package-{}-{}",
    std::process::id(),
    NEXT.fetch_add(1, Ordering::Relaxed)
  ));
  let package = root.join("node_modules/tailwindcss");
  fs::create_dir_all(&package).expect("create isolated package fixture");
  fs::write(
    package.join("index.css"),
    r#"
@layer theme, base, utilities;
@layer theme { @theme default { --color-brand: #123456; } }
@layer base { *, ::before, ::after { margin: 0; padding: 0; border: 0 solid; } }
@layer utilities { @tailwind utilities; }
"#,
  )
  .expect("write package stylesheet");
  let mut compiler = compile(
    "@import 'tailwindcss';",
    CompileOptions {
      base: root.to_string_lossy().into_owned(),
      ..Default::default()
    },
  )
  .expect("compile package import");
  let css = compiler.build(&["bg-brand".to_string(), "md:grid-cols-2".to_string()]);
  assert!(css.contains("@layer base"), "{css}");
  assert!(css.contains("border: 0 solid"), "{css}");
  assert!(css.contains("--color-brand: #123456"), "{css}");
  assert!(css.contains(".md\\:grid-cols-2"), "{css}");
  assert!(compiler
    .dependencies()
    .iter()
    .any(|path| path.ends_with("index.css")));
  fs::remove_dir_all(&root).expect("remove owned fixture directory");
}
