use std::collections::BTreeMap;

use anyhow::{Context, Result};

use crate::DomainFrame;

pub fn insert_frame(map: &mut BTreeMap<String, DomainFrame>, frame: DomainFrame) -> Result<()> {
    let key = frame.domain_code.to_uppercase();
    if let Some(existing) = map.get_mut(&key) {
        existing
            .data
            .vstack_mut(&frame.data)
            .with_context(|| format!("merge {key} frames"))?;
        // Merge source files from the incoming frame
        if let Some(ref meta) = frame.meta {
            for source in &meta.source_files {
                existing.add_source_file(source.clone());
            }
        }
    } else {
        map.insert(
            key.clone(),
            DomainFrame {
                domain_code: key.clone(),
                data: frame.data,
                meta: frame.meta,
            },
        );
    }
    Ok(())
}
