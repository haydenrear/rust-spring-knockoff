use std::collections::{HashMap, HashSet};
use std::{env, fs};
use std::path::PathBuf;
use cargo::Config;
use cargo::core::{Package, PackageId, PackageSet, SourceId, SourceMap};
use cargo::sources::RegistrySource;
use cargo::util::IntoUrl;

pub fn download_cargo_crate<'a>(name: &str, version: &str, source_id: &SourceId, config: Config) -> Option<Package> {

    let whitelist = HashSet::new();
    let mut source = RegistrySource::remote(source_id.clone(), &whitelist, &config).unwrap();
    let mut source_map = SourceMap::new();

    source_map.insert(Box::new(source));

    let package_id = PackageId::new(name, version, source_id.clone()).unwrap();

    let p = PackageSet::new(
        &[package_id.clone()], source_map, &config
    ).unwrap();

    let mut downloaded = p.enable_download().unwrap();
    downloaded.start(package_id)
        .expect("Could not start downloading package");
    while downloaded.remaining() != 0 {
        downloaded.wait()
            .expect("Could not wait for package");
    }
    let package = p.get_one(package_id).ok().cloned();
    package
}

pub fn download_cargo_crate_to_directory(
    registry_id: &SourceId,
    module_macro_lib: &str,
    config: Config,
    args: &HashMap<String, String>,
    target_directory: PathBuf
) {
    let version = get_version(args);
    download_cargo_crate(module_macro_lib, &version, registry_id, config)
        .map(|pkg|
            copy_cargo_crate(
                target_directory.join(module_macro_lib),
                pkg
            ));
}

pub fn get_version(args: &HashMap<String, String>) -> String {
    let fallback_version = "0.1.5".to_string();
    let version = args.get(&"version".to_string())
        .or(Some(&fallback_version)).unwrap();
    version.clone()
}

pub fn copy_cargo_crate(target_pkg_path: PathBuf, registry_pkg: Package) {
    let registry_pkg_path = registry_pkg.root();
    fs::create_dir_all(&target_pkg_path).unwrap();
    codegen_utils::copy_dir(&registry_pkg_path, &target_pkg_path).unwrap();
}


pub fn get_registry_source_id(registry_uri: Option<&String>) -> SourceId {
    registry_uri
        .or(env::var("MODULE_MACRO_REGISTRY_INDEX_URI").ok().as_ref())
        .map(|registry_uri| SourceId::for_registry(
            &registry_uri.into_url().unwrap()
        ).ok()
        )
        .flatten()
        .or(Some(SourceId::crates_io(&Config::default().unwrap()).unwrap()))
        .unwrap()
}
