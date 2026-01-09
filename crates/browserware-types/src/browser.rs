use std::path::PathBuf;

use super::{BrowserFamily, Channel};

pub struct Browser {
    pub name: String,
    pub family: BrowserFamily,
    pub channel: Channel,
    pub executable: PathBuf,
}
