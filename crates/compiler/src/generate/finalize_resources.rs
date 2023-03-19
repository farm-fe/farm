use std::sync::Arc;

use farmfe_core::context::CompilationContext;
use farmfe_toolkit::tracing;

#[tracing::instrument(skip_all)]
pub fn finalize_resources(context: &Arc<CompilationContext>) -> farmfe_core::error::Result<()> {
  tracing::trace!("Staring finalize_resources...");

  {
    let mut resources_map = context.resources_map.lock();

    context
      .plugin_driver
      .finalize_resources(&mut resources_map, context)?;
  }

  // write_resources(context);

  tracing::trace!("Finished finalize_resources.");
  Ok(())
}

// fn write_resources(context: &Arc<CompilationContext>) {
//   let resources = context.resources_map.lock();
//   let output_dir = if Path::new(&context.config.output.path).is_absolute() {
//     PathBuf::from(&context.config.output.path)
//   } else {
//     RelativePath::new(&context.config.output.path).to_logical_path(&context.config.root)
//   };

//   if !output_dir.exists() {
//     create_dir_all(output_dir.clone()).unwrap();
//   }

//   // Remove useless resources
//   // TODO support sub dir scene
//   let existing_resources = read_dir(output_dir.clone())
//     .unwrap()
//     .map(|entry| {
//       let entry = entry.unwrap();
//       let path = entry.path();
//       let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

//       file_name
//     })
//     .collect::<Vec<String>>();

//   for pre_resource in &existing_resources {
//     let file_path = RelativePath::new(pre_resource).to_logical_path(&output_dir);
//     // always remove html file
//     if pre_resource.ends_with(".html") {
//       remove_file(file_path).unwrap();
//       continue;
//     }

//     if !resources.contains_key(pre_resource) && file_path.exists() {
//       remove_file(file_path).unwrap();
//     }
//   }

//   // add new resources
//   let resources_vec = resources
//     .values()
//     .collect::<Vec<&farmfe_core::resource::Resource>>();
//   resources_vec.into_par_iter().for_each(|resource| {
//     let file_path = RelativePath::new(&resource.name).to_logical_path(&output_dir);
//     // only write expose non-emitted resource
//     if !resource.emitted && !file_path.exists() {
//       let mut file = File::create(file_path).unwrap();

//       file.write_all(&resource.bytes).unwrap();
//       file.sync_data().unwrap();
//     }
//   });
// }
