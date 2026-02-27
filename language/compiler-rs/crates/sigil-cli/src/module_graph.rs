//! Module graph construction and dependency resolution
//!
//! Handles building a dependency graph of Sigil modules for multi-module compilation

use crate::project::ProjectConfig;
use sigil_ast::Program;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ModuleGraph {
    pub modules: HashMap<String, LoadedModule>,
    pub topo_order: Vec<String>,
}

pub struct LoadedModule {
    pub id: String,
    pub file_path: PathBuf,
    pub source: String,
    pub ast: Program,
    pub project: Option<ProjectConfig>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            topo_order: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: LoadedModule) {
        let id = module.id.clone();
        self.modules.insert(id.clone(), module);
        self.topo_order.push(id);
    }
}
