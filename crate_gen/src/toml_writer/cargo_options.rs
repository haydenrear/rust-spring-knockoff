use toml::{Table, Value};
use std::path::PathBuf;

pub struct CargoTomlOptions<'a> {
    pub(crate) workspace: bool,
    pub(crate) dependencies: Option<&'a Table>,
    pub(crate) build_dependencies: Option<&'a Table>,
    pub(crate) name: Option<&'a str>,
    pub(crate) version: Option<&'a str>,
    pub(crate) target_path: Option<PathBuf>
}

impl<'a> CargoTomlOptions<'a> {

    pub fn open_options() -> Self {
        Self {
            workspace: false,
            dependencies: None,
            build_dependencies: None,
            name: None,
            version: None,
            target_path: None
        }
    }

    pub fn get_dependencies(&mut self, default_value: &'a Table) -> &'a Table {
        if self.dependencies.is_none() {
            self.dependencies = Some(default_value);
            return self.dependencies.unwrap();
        }
        self.dependencies.unwrap()
    }

    pub fn get_build_dependencies(&mut self, default_value: &'a Table) -> &'a Table {
        if self.dependencies.is_none() {
            self.build_dependencies = Some(default_value);
            return self.build_dependencies.unwrap()
        }
        self.build_dependencies.unwrap()
    }

    pub fn target_path(&mut self, add_workspace: PathBuf) -> &mut Self {
        self.target_path = Some(add_workspace);
        self
    }

    pub fn workspace(&mut self, add_workspace: bool) -> &mut Self {
        self.workspace = add_workspace;
        self
    }

    pub fn build_dependencies(&mut self, deps: &'a Table) -> &mut Self {
        self.build_dependencies = Some(deps);
        self
    }

    pub fn dependencies(&mut self, deps: &'a Table) -> &mut Self {
        self.dependencies = Some(deps);
        self
    }

    pub fn version(&'a mut self, version_val: &'a str) -> &mut Self {
        self.version = Some(version_val);
        self
    }

    pub fn name(&'a mut self, name: &'a str) -> &mut Self {
        self.name = Some(name);
        self
    }

}
