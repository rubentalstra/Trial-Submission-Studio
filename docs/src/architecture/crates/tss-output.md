# tss-output

Multi-format export crate.

## Overview

`tss-output` generates output files in XPT, Dataset-XML, and Define-XML formats.

## Responsibilities

- Coordinate export to multiple formats
- Generate XPT files (via tss-xpt)
- Generate Dataset-XML
- Generate Define-XML 2.1
- Create checksums

## Dependencies

```toml
[dependencies]
quick-xml = "0.36"
tss-xpt = { path = "../tss-xpt" }
tss-model = { path = "../tss-model" }
tss-standards = { path = "../tss-standards" }
sha2 = "0.10"
```

## Architecture

### Module Structure

```
tss-output/
├── src/
│   ├── lib.rs
│   ├── exporter.rs      # Export orchestration
│   ├── xpt.rs           # XPT export wrapper
│   ├── dataset_xml.rs   # Dataset-XML generation
│   ├── define_xml.rs    # Define-XML generation
│   └── checksum.rs      # SHA256 generation
```

## Export Formats

### XPT Export

Delegates to `tss-xpt`:

```rust
pub fn export_xpt(
    data: &DataFrame,
    metadata: &DatasetMetadata,
    path: &Path,
    version: XptVersion,
) -> Result<()> {
    let writer = XptWriter::new(path, version)?;
    writer.write_metadata(metadata)?;
    writer.write_data(data)?;
    writer.finish()
}
```

### Dataset-XML Export

```rust
pub fn export_dataset_xml(
    data: &DataFrame,
    metadata: &DatasetMetadata,
    path: &Path,
) -> Result<()> {
    let mut writer = XmlWriter::new(path)?;
    writer.write_odm_header()?;
    writer.write_clinical_data(data, metadata)?;
    writer.finish()
}
```

### Define-XML Export

```rust
pub fn export_define_xml(
    datasets: &[DatasetMetadata],
    standards: &Standards,
    path: &Path,
) -> Result<()> {
    let mut writer = DefineXmlWriter::new(path)?;
    writer.write_study_metadata()?;
    writer.write_item_group_defs(datasets)?;
    writer.write_item_defs(datasets)?;
    writer.write_codelists()?;
    writer.finish()
}
```

## API

### Single Dataset Export

```rust
use tss_output::{Exporter, ExportOptions, ExportFormat};

let exporter = Exporter::new();
let options = ExportOptions {
format: ExportFormat::XptV5,
generate_checksum: true,
};

exporter.export( & data, & metadata, "dm.xpt", options) ?;
```

### Batch Export

```rust
let batch_options = BatchExportOptions {
output_dir: PathBuf::from("./output"),
formats: vec![ExportFormat::XptV5, ExportFormat::DefineXml],
generate_checksums: true,
};

exporter.export_batch( & datasets, batch_options) ?;
```

## Checksum Generation

```rust
pub fn generate_checksum(path: &Path) -> Result<String> {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    let mut file = File::open(path)?;
    std::io::copy(&mut file, &mut hasher)?;

    Ok(format!("{:x}", hasher.finalize()))
}
```

Output: `dm.xpt.sha256` containing:

```
abc123...def456  dm.xpt
```

## Testing

```bash
cargo test --package tss-output
```

### Test Strategy

- Output format validation
- Roundtrip testing (export then read)
- Checksum verification
- Define-XML schema validation

## See Also

- [Exporting Data](../../user-guide/exporting-data.md) - User guide
- [XPT Format](../../output-formats/xpt-format.md) - XPT details
- [Define-XML](../../output-formats/define-xml.md) - Define-XML details
- [tss-xpt](tss-xpt.md) - XPT implementation
