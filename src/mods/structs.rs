use serde_derive::{Deserialize};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Manifest {
    name: String,
    author: String,
    pck_info: Option<PckInfo>,
    patches: Vec<Patch>,
    deps: Option<Vec<String>>,

    // Reserved
    #[serde(skip_serializing)]
    #[serde(default)]
    pub path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Patch {
    resource: String,
    patch_file: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PckInfo {
    directory: String,
    resource_prefix: String
}


impl Manifest {
    pub fn get_dependencies(&self) -> Vec<String> {
        match self.deps {
            Some(ref deps) => deps.clone(),
            None => Vec::new()
        }
    }

    pub fn get_author(&self) -> String {
        self.author.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_patches(&self) -> &Vec<Patch> {
        &self.patches
    }

    pub fn get_pck_info(&self) -> &Option<PckInfo> {
        &self.pck_info
    }
}

impl Patch {
    pub fn get_resource(&self) -> String {
        self.resource.clone()
    }

    pub fn get_patch_file(&self) -> String {
        self.patch_file.clone()
    }
}

impl PckInfo {
    pub fn get_directory(&self) -> String {
        self.directory.clone()
    }

    pub fn get_resource_prefix(&self) -> String {
        self.resource_prefix.clone()
    }
}