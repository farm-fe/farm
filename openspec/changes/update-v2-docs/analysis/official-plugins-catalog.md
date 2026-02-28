# Farm v2 Official Plugins Catalog

## JavaScript Plugins (`js-plugins/`)

### CSS & Styling Plugins

1. **@farmfe/js-plugin-sass** (`js-plugins/sass/`)
   - Sass/SCSS preprocessor support
   - Location: `js-plugins/sass/`

2. **@farmfe/js-plugin-less** (`js-plugins/less/`)
   - Less preprocessor support
   - Location: `js-plugins/less/`

3. **@farmfe/js-plugin-postcss** (`js-plugins/postcss/`)
   - PostCSS processing support
   - Location: `js-plugins/postcss/`

4. **@farmfe/js-plugin-tailwindcss** (`js-plugins/tailwindcss/`)
   - Tailwind CSS framework support
   - Location: `js-plugins/tailwindcss/`

### Framework Plugins

5. **@farmfe/js-plugin-vue** (`js-plugins/vue/`)
   - Vue.js framework support
   - Location: `js-plugins/vue/`

6. **@farmfe/js-plugin-solid** (`js-plugins/solid/`)
   - Solid.js framework support
   - Location: `js-plugins/solid/`

### Tooling Plugins

7. **@farmfe/js-plugin-dts** (`js-plugins/dts/`)
   - TypeScript declaration file generation
   - Location: `js-plugins/dts/`

8. **@farmfe/js-plugin-svgr** (`js-plugins/svgr/`)
   - SVG to React component transformation
   - Location: `js-plugins/svgr/`

9. **@farmfe/js-plugin-visualizer** (`js-plugins/visualizer/`)
   - Bundle visualization and analysis
   - Location: `js-plugins/visualizer/`

10. **@farmfe/js-plugin-electron** (`js-plugins/electron/`)
    - Electron application support
    - Location: `js-plugins/electron/`

11. **@farmfe/js-plugin-record-viewer** (`js-plugins/record-viewer/`)
    - Build record viewing
    - Location: `js-plugins/record-viewer/`

## Rust Plugins (`rust-plugins/`)

1. **react** (`rust-plugins/react/`)
   - React framework support (Rust implementation)
   - Location: `rust-plugins/react/`

2. **sass** (`rust-plugins/sass/`)
   - Sass/SCSS preprocessor (Rust implementation)
   - Location: `rust-plugins/sass/`

3. **dts** (`rust-plugins/dts/`)
   - TypeScript declaration generation (Rust implementation)
   - Location: `rust-plugins/dts/`

4. **replace-dirname** (`rust-plugins/replace-dirname/`)
   - __dirname replacement utility
   - Location: `rust-plugins/replace-dirname/`

## Internal Rust Plugins (`crates/plugin_*`)

These are built-in plugins compiled into Farm core:

### Module Processing

1. **plugin_script** - JavaScript/TypeScript module handling
2. **plugin_script_meta** - Script metadata extraction
3. **plugin_css** - CSS module handling
4. **plugin_json** - JSON module support
5. **plugin_html** - HTML processing
6. **plugin_static_assets** - Static asset handling

### Build Optimization

7. **plugin_tree_shake** - Tree shaking optimization
8. **plugin_minify** - Code minification
9. **plugin_mangle_exports** - Export name mangling
10. **plugin_partial_bundling** - Partial bundling strategy

### Development

11. **plugin_lazy_compilation** - Lazy compilation for dev
12. **plugin_progress** - Build progress reporting
13. **plugin_define** - Build-time constant definition
14. **plugin_polyfill** - Polyfill injection

### Output

15. **plugin_runtime** - Runtime code generation
16. **plugin_library** - Library mode support
17. **plugin_resolve** - Module resolution
18. **plugin_file_size** - File size reporting

## Plugin Categories Summary

### By Language
- **JavaScript Plugins**: 11 plugins
- **Rust Plugins (external)**: 4 plugins  
- **Internal Rust Plugins**: 18 plugins
- **Total**: 33 plugins

### By Function
- **CSS/Styling**: 4 plugins (sass x2, less, postcss, tailwindcss)
- **Frameworks**: 3 plugins (vue, solid, react)
- **TypeScript**: 2 plugins (dts x2)
- **Assets**: 2 plugins (svgr, static_assets)
- **Optimization**: 4 plugins (tree_shake, minify, mangle_exports, partial_bundling)
- **Development**: 3 plugins (lazy_compilation, progress, electron)
- **Module Handling**: 4 plugins (script, script_meta, json, html)
- **Utilities**: 6 plugins (visualizer, record-viewer, define, polyfill, resolve, file_size)
- **Output/Runtime**: 2 plugins (runtime, library)
- **Other**: 3 plugins (replace-dirname, css, etc.)

## Documentation Impact

Each of these plugins needs to be accurately documented based on their actual implementation:

### High Priority (User-facing)
- CSS preprocessors (sass, less, postcss, tailwindcss)
- Framework plugins (react, vue, solid)
- TypeScript tools (dts)
- Development tools (lazy_compilation, progress)

### Medium Priority (Configuration)
- Build optimization (tree_shake, minify, partial_bundling)
- Asset handling (static_assets, svgr)
- Module processing (script, css, html, json)

### Low Priority (Internal/Advanced)
- Internal runtime plugins
- Advanced optimization plugins
- Utility plugins
