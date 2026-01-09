use browserware_types::BrowserFamily;
use std::path::Path;
use url::Url;

pub fn launch(_executable: &Path, _family: BrowserFamily, _urls: &[Url]) -> anyhow::Result<()> {
    todo!()
}
