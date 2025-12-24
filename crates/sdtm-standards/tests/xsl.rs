use sdtm_standards::list_default_xsl_assets;

#[test]
fn lists_xsl_assets() {
    let assets = list_default_xsl_assets();
    assert!(!assets.is_empty());
    assert!(assets.iter().any(|asset| asset.name.contains("define")));
}
