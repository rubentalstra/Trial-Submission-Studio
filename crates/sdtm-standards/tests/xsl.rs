use std::path::Path;

use sdtm_standards::list_xsl_assets;

#[test]
fn lists_xsl_assets() {
    let dir = Path::new("../../standards/xsl");
    let assets = list_xsl_assets(dir);
    assert!(!assets.is_empty());
    assert!(assets.iter().any(|asset| asset.name.contains("define")));
}
