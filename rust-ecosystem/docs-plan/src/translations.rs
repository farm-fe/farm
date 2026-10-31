use crate::{routes, DocPlan, PageAlternate, PageSource};
use farmfe_docs_config::{ResolvedCollection, ValidatedConfig};
use std::collections::{BTreeMap, BTreeSet, HashMap};

fn groups(config: &ValidatedConfig) -> BTreeMap<String, Vec<&ResolvedCollection>> {
  let mut groups: BTreeMap<String, Vec<&ResolvedCollection>> = BTreeMap::new();
  for collection in &config.collections {
    if let Some(group) = &collection.public.group {
      groups
        .entry(format!("{:?}:{group}", collection.public.kind))
        .or_default()
        .push(collection);
    }
  }
  groups
}
pub fn expand(config: &ValidatedConfig, plan: &mut DocPlan) -> Result<(), String> {
  let groups = groups(config);
  let mut index: HashMap<_, _> = plan
    .routes
    .iter()
    .enumerate()
    .map(|(index, route)| {
      (
        (
          route.page.collection.clone(),
          routes::logical_id(&route.page.id),
        ),
        index,
      )
    })
    .collect();
  let originals = index.clone();
  let mut urls: HashMap<_, _> = plan
    .routes
    .iter()
    .zip(&plan.modules)
    .map(|(route, module)| {
      (
        routes::collision_key(&route.page.route_path),
        module.source_path.clone(),
      )
    })
    .collect();
  for (group, collections) in &groups {
    let locales: BTreeSet<_> = collections
      .iter()
      .map(|collection| collection.public.locale.as_str())
      .collect();
    let versions: BTreeSet<_> = collections
      .iter()
      .map(|collection| collection.public.version.clone())
      .collect();
    for version in &versions {
      for locale in &locales {
        if !collections.iter().any(|collection| {
          &collection.public.version == version && collection.public.locale == *locale
        }) {
          return Err(format!(
            "[DOCS_COLLECTION_MATRIX] group {group}: missing locale {locale}, version {version:?}"
          ));
        }
      }
      let members: Vec<_> = collections
        .iter()
        .filter(|collection| &collection.public.version == version)
        .collect();
      let ids: BTreeSet<_> = plan
        .routes
        .iter()
        .filter(|route| {
          members
            .iter()
            .any(|collection| collection.public.id == route.page.collection)
        })
        .map(|route| routes::logical_id(&route.page.id))
        .collect();
      for id in &ids {
        for target in &members {
          let slot = (target.public.id.clone(), id.clone());
          if index.contains_key(&slot) {
            continue;
          }
          let source = target.fallback_locale.as_ref().and_then(|locale| {
            members
              .iter()
              .find(|collection| collection.public.locale == *locale)
          });
          let source_index = source.and_then(|source| {
            originals
              .get(&(source.public.id.clone(), id.clone()))
              .copied()
          });
          let Some((source, source_index)) = source.zip(source_index) else {
            return Err(format!(
              "[DOCS_TRANSLATION_MISSING] group {group}, id {id:?}: collection {} ({}, {version:?}) requires a real translation or explicit fallback with an existing source",
              target.public.id,
              target.directory.display()
            ));
          };
          let mut route = plan.routes[source_index].clone();
          let mut module = plan.modules[source_index].clone();
          let prefix = format!(
            "{}{}",
            config.public.base_path,
            source.public.route_base.trim_start_matches('/')
          );
          let suffix = route
            .page
            .route_path
            .strip_prefix(&prefix)
            .ok_or("[DOCS_ROUTE_PATH] source collection prefix mismatch")?
            .to_string();
          route.page.fallback_source = Some(PageSource::from(&route.page));
          route.page.key = routes::page_key(&target.public, id)?;
          route.page.collection = target.public.id.clone();
          route.page.locale = target.public.locale.clone();
          route.page.version = target.public.version.clone();
          route.page.route_path = format!(
            "{}{}{}",
            config.public.base_path,
            target.public.route_base.trim_start_matches('/'),
            suffix
          );
          route.output_path = format!(
            "{}index.html",
            route
              .page
              .route_path
              .strip_prefix(&config.public.base_path)
              .ok_or("[DOCS_ROUTE_PATH] target base prefix mismatch")?
          );
          routes::validate_output(&route.output_path, &module.source_path)?;
          if let Some(previous) = urls.insert(
            routes::collision_key(&route.page.route_path),
            module.source_path.clone(),
          ) {
            return Err(format!(
              "[DOCS_ROUTE_COLLISION] fallback {}: {previous} and {}",
              route.page.route_path, module.source_path
            ));
          }
          module.key = route.page.key.clone();
          index.insert(slot, plan.routes.len());
          plan.routes.push(route);
          plan.modules.push(module);
        }
      }
    }
  }
  Ok(())
}
pub fn connect(config: &ValidatedConfig, plan: &mut DocPlan) -> Result<(), String> {
  let groups = groups(config);
  let index: HashMap<_, _> = plan
    .routes
    .iter()
    .enumerate()
    .map(|(index, route)| {
      (
        (
          route.page.collection.clone(),
          routes::logical_id(&route.page.id),
        ),
        index,
      )
    })
    .collect();
  let mut homes = HashMap::new();
  for collection in config
    .collections
    .iter()
    .filter(|collection| collection.public.group.is_some())
  {
    let root = format!(
      "{}{}",
      config.public.base_path,
      collection.public.route_base.trim_start_matches('/')
    );
    let home = plan
      .routes
      .iter()
      .filter(|route| route.page.collection == collection.public.id)
      .min_by_key(|route| {
        (
          route.page.route_path != routes::public_path(&root, &config.public),
          &route.page.route_path,
        )
      })
      .ok_or_else(|| {
        format!(
          "[DOCS_COLLECTION_EMPTY] collection {} has no output page for navigation",
          collection.public.id
        )
      })?;
    homes.insert(collection.public.id.clone(), home.page.route_path.clone());
  }
  // Resolve every alternate only after all genuine and explicit fallback pages exist.
  for position in 0..plan.routes.len() {
    let page = &plan.routes[position].page;
    let Some(collection) = config
      .collections
      .iter()
      .find(|collection| collection.public.id == page.collection)
    else {
      return Err("[DOCS_INDEX] missing collection".into());
    };
    let Some(group) = &collection.public.group else {
      continue;
    };
    let members = groups
      .get(&format!("{:?}:{group}", collection.public.kind))
      .ok_or("[DOCS_INDEX] missing logical group")?;
    let id = routes::logical_id(&page.id);
    let alternates = members
      .iter()
      .map(|target| {
        let home = homes
          .get(&target.public.id)
          .ok_or("[DOCS_INDEX] missing collection home")?;
        let matched = index
          .get(&(target.public.id.clone(), id.clone()))
          .map(|index| &plan.routes[*index].page);
        Ok(PageAlternate {
          collection: target.public.id.clone(),
          locale: target.public.locale.clone(),
          version: target.public.version.clone(),
          route_path: matched
            .map(|page| page.route_path.clone())
            .unwrap_or_else(|| home.clone()),
          key: matched.map(|page| page.key.clone()),
          missing: matched.is_none(),
        })
      })
      .collect::<Result<Vec<_>, String>>()?;
    plan.routes[position].page.alternates = alternates;
  }
  Ok(())
}
