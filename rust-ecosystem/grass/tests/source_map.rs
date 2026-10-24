use grass_compiler::{from_path_with_source_map, Options, OutputStyle};

#[test]
fn maps_dotted_imports_unicode_and_generated_values_to_original_sources() {
    let root = std::env::temp_dir().join(format!("farm-grass-map-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&root).unwrap();
    let main = root.join("main.scss");
    let source = "// 世界\n@use './part.name' as part;\n.box {\ncontent: '世界🌱';\npadding: 7px;\n}\n";
    std::fs::write(&main, source).unwrap();
    std::fs::write(root.join("_part.name.scss"), ".imported { margin: 13px; }\n").unwrap();
    for style in [OutputStyle::Expanded, OutputStyle::Compressed] {
        let result = from_path_with_source_map(&main, &Options::default().style(style)).unwrap();
        assert_eq!(result.css, grass_compiler::from_path(&main, &Options::default().style(style)).unwrap());
        let map = sourcemap::SourceMap::from_slice(result.source_map.as_ref().unwrap().as_bytes()).unwrap();
        for (value, filename, original_line) in [("7px", "main.scss", 4), ("13px", "_part.name.scss", 0)] {
            let offset = result.css.find(value).unwrap();
            let prefix = &result.css[..offset];
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
            let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
            let token = map.lookup_token(line, column).unwrap();
            assert!(token.get_source().unwrap().ends_with(filename));
            assert_eq!(token.get_src_line(), original_line);
            let source_line = if filename == "main.scss" { "padding: 7px;" } else { ".imported { margin: 13px; }" };
            assert_eq!(token.get_src_col(), source_line[..source_line.find(value).unwrap()].encode_utf16().count() as u32);
        }
        assert!(map.source_contents().any(|content| content == Some(source)));
    }
    std::fs::remove_dir_all(root).unwrap();
}
