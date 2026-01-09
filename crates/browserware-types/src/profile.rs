use std::path::PathBuf;

pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub is_default: bool,
}
