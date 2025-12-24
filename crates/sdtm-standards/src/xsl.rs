use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct XslAsset {
    pub name: String,
    pub path: PathBuf,
}

pub fn list_xsl_assets(xsl_dir: &Path) -> Vec<XslAsset> {
    let mut assets = Vec::new();
    if !xsl_dir.exists() {
        return assets;
    }
    let entries = match std::fs::read_dir(xsl_dir) {
        Ok(entries) => entries,
        Err(_) => return assets,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|v| v.to_str()).unwrap_or("") != "xsl" {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_string();
        assets.push(XslAsset { name, path });
    }
    assets.sort_by(|a, b| a.name.cmp(&b.name));
    assets
}
