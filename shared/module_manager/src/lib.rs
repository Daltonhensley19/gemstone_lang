use std::{ops::IndexMut, path::PathBuf};

#[derive(Debug)]
pub struct Module {
    pub src: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct ModuleManager {
    modules: Vec<Module>,
}

impl ModuleManager {
    pub fn new() -> Result<Self, std::io::Error> {
        // Get root directory for `modules`
        let modules_path = "modules/";
        let dir = std::fs::read_dir(modules_path)?;

        // Get the paths to the modules
        let module_paths = dir
            .into_iter()
            .filter_map(|f| f.ok().map(|f| f.path().display().to_string()))
            .collect::<Vec<String>>();

        // Get the actual file contents of these modules from their `modules_paths`
        let module_srcs = module_paths
            .iter()
            .map(|f| std::fs::read_to_string(f).unwrap())
            .collect::<Vec<String>>();

        let modules = module_paths
            .into_iter()
            .zip(module_srcs)
            .map(|(f, src)| Module {
                src,
                path: PathBuf::from(f),
            })
            .collect::<Vec<Module>>();

        Ok(ModuleManager { modules })
    }

    pub fn get_ref(&self) -> &Vec<Module> {
        &self.modules
    }

    pub fn get_mut_ref(&mut self) -> &mut Vec<Module> {
        &mut self.modules
    }
}
