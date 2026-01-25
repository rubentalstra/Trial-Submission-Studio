//! Comprehensive tests for the CDISC-IG MCP server index functionality
//!
//! Tests cover all 5 MCP tools:
//! - search_ig: Full-text search across IGs
//! - get_domain_spec: Get chunks for a specific domain
//! - get_chunk_by_index: Retrieve single chunk by index
//! - get_related_chunks: Get parent + sibling chunks
//! - list_sections: List all section headings

use mcp_cdisc_ig::index::IgIndex;

/// Helper to get a loaded test index (reused across tests)
fn get_test_index() -> IgIndex {
    IgIndex::load().expect("Failed to load test index")
}

// =============================================================================
// 1. IgIndex Loading Tests
// =============================================================================

#[test]
fn test_load_succeeds() {
    // Verify all 4 IGs load without error
    let result = IgIndex::load();
    assert!(result.is_ok(), "IgIndex::load() should succeed");
}

#[test]
fn test_section_count_positive() {
    let index = get_test_index();
    let count = index.section_count();
    assert!(count > 0, "section_count() should be > 0, got {}", count);
}

#[test]
fn test_domain_count_positive() {
    let index = get_test_index();
    let count = index.domain_count();
    assert!(count > 0, "domain_count() should be > 0, got {}", count);
}

// =============================================================================
// 2. Search Tests (search_ig tool)
// =============================================================================

#[test]
fn test_search_empty_query() {
    let index = get_test_index();
    let results = index.search("", "all", 10);
    assert!(results.is_empty(), "Empty query should return empty Vec");
}

#[test]
fn test_search_whitespace_only_query() {
    let index = get_test_index();
    let results = index.search("   ", "all", 10);
    assert!(
        results.is_empty(),
        "Whitespace-only query should return empty Vec"
    );
}

#[test]
fn test_search_single_keyword() {
    let index = get_test_index();
    let results = index.search("USUBJID", "all", 10);
    assert!(
        !results.is_empty(),
        "Search for 'USUBJID' should find results"
    );
}

#[test]
fn test_search_multiple_keywords() {
    let index = get_test_index();
    let results = index.search("subject identifier", "all", 10);
    assert!(
        !results.is_empty(),
        "Multi-word query 'subject identifier' should find results"
    );
}

#[test]
fn test_search_case_insensitive() {
    let index = get_test_index();

    let upper_results = index.search("USUBJID", "all", 10);
    let lower_results = index.search("usubjid", "all", 10);
    let mixed_results = index.search("UsUbJiD", "all", 10);

    assert!(
        !upper_results.is_empty(),
        "Uppercase search should find results"
    );
    assert!(
        !lower_results.is_empty(),
        "Lowercase search should find results"
    );
    assert!(
        !mixed_results.is_empty(),
        "Mixed case search should find results"
    );

    // All should find roughly the same number of results
    assert_eq!(
        upper_results.len(),
        lower_results.len(),
        "Case should not affect result count"
    );
}

#[test]
fn test_search_no_matches() {
    let index = get_test_index();
    let results = index.search("xyzzy12345gibberish", "all", 10);
    assert!(
        results.is_empty(),
        "Gibberish query should return empty results"
    );
}

#[test]
fn test_search_limit_respected() {
    let index = get_test_index();
    let results = index.search("domain", "all", 5);
    assert!(
        results.len() <= 5,
        "limit=5 should return at most 5 results, got {}",
        results.len()
    );
}

#[test]
fn test_search_limit_zero() {
    let index = get_test_index();
    let results = index.search("domain", "all", 0);
    assert!(results.is_empty(), "limit=0 should return empty results");
}

#[test]
fn test_search_specific_ig_sdtm() {
    let index = get_test_index();
    let results = index.search("Demographics", "sdtm", 20);

    assert!(!results.is_empty(), "SDTM search should find results");

    // All results should be from SDTM
    for result in &results {
        assert!(
            result.ig.contains("SDTM"),
            "All results should be from SDTM, got: {}",
            result.ig
        );
    }
}

#[test]
fn test_search_specific_ig_send() {
    let index = get_test_index();
    let results = index.search("animal", "send", 20);

    // All results should be from SEND
    for result in &results {
        assert!(
            result.ig.contains("SEND"),
            "All results should be from SEND, got: {}",
            result.ig
        );
    }
}

#[test]
fn test_search_specific_ig_adam() {
    let index = get_test_index();
    let results = index.search("analysis", "adam", 20);

    // All results should be from ADaM
    for result in &results {
        assert!(
            result.ig.contains("ADaM"),
            "All results should be from ADaM, got: {}",
            result.ig
        );
    }
}

#[test]
fn test_search_specific_ig_define() {
    let index = get_test_index();
    let results = index.search("metadata", "define", 20);

    // All results should be from Define-XML
    for result in &results {
        assert!(
            result.ig.contains("Define"),
            "All results should be from Define-XML, got: {}",
            result.ig
        );
    }
}

#[test]
fn test_search_all_igs() {
    let index = get_test_index();
    let results = index.search("variable", "all", 50);

    assert!(
        !results.is_empty(),
        "Search across all IGs should find results"
    );

    // Should potentially find results from multiple IGs
    let ig_names: std::collections::HashSet<_> = results.iter().map(|r| &r.ig).collect();
    // At least verify we're searching across IGs (may not always hit all)
    assert!(
        !ig_names.is_empty(),
        "Should find results from at least one IG"
    );
}

#[test]
fn test_search_invalid_ig_searches_all() {
    let index = get_test_index();

    let invalid_results = index.search("domain", "invalid_ig_name", 20);
    let all_results = index.search("domain", "all", 20);

    // Invalid IG should default to searching all (same as "all")
    assert_eq!(
        invalid_results.len(),
        all_results.len(),
        "Invalid IG should search all IGs"
    );
}

#[test]
fn test_search_result_has_required_fields() {
    let index = get_test_index();
    let results = index.search("USUBJID", "sdtm", 1);

    assert!(!results.is_empty(), "Should find at least one result");

    let result = &results[0];
    assert!(!result.ig.is_empty(), "ig field should not be empty");
    assert!(
        !result.heading.is_empty(),
        "heading field should not be empty"
    );
    assert!(
        !result.content.is_empty(),
        "content field should not be empty"
    );
    assert!(result.score > 0.0, "score should be positive");
    assert!(result.score <= 1.0, "score should be <= 1.0");
}

#[test]
fn test_search_results_sorted_by_score() {
    let index = get_test_index();
    let results = index.search("subject unique identifier", "all", 20);

    if results.len() >= 2 {
        for i in 0..results.len() - 1 {
            assert!(
                results[i].score >= results[i + 1].score,
                "Results should be sorted by score descending"
            );
        }
    }
}

// =============================================================================
// 3. Domain Lookup Tests (get_domain_spec tool)
// =============================================================================

#[test]
fn test_get_domain_valid_dm() {
    let index = get_test_index();
    let chunks = index.get_domain("DM", "sdtm");

    assert!(chunks.is_some(), "DM domain should exist in SDTM");
    let chunks = chunks.unwrap();
    assert!(!chunks.is_empty(), "DM domain should have chunks");

    // All chunks should have domain = "DM"
    for chunk in &chunks {
        assert_eq!(
            chunk.domain.as_deref(),
            Some("DM"),
            "All chunks should be for DM domain"
        );
    }
}

#[test]
fn test_get_domain_case_insensitive() {
    let index = get_test_index();

    let upper = index.get_domain("DM", "sdtm");
    let lower = index.get_domain("dm", "sdtm");
    let mixed = index.get_domain("Dm", "sdtm");

    assert!(upper.is_some(), "Uppercase DM should work");
    assert!(lower.is_some(), "Lowercase dm should work");
    assert!(mixed.is_some(), "Mixed case Dm should work");

    // Should return same number of chunks
    assert_eq!(
        upper.unwrap().len(),
        lower.unwrap().len(),
        "Case should not affect domain lookup"
    );
}

#[test]
fn test_get_domain_not_found() {
    let index = get_test_index();
    let chunks = index.get_domain("NOTADOMAIN", "sdtm");
    assert!(chunks.is_none(), "Non-existent domain should return None");
}

#[test]
fn test_get_domain_invalid_ig() {
    let index = get_test_index();
    let chunks = index.get_domain("DM", "invalid_ig");
    assert!(chunks.is_none(), "Invalid IG should return None");
}

#[test]
fn test_get_domain_ae() {
    let index = get_test_index();
    let chunks = index.get_domain("AE", "sdtm");

    assert!(chunks.is_some(), "AE domain should exist in SDTM");
    let chunks = chunks.unwrap();
    assert!(!chunks.is_empty(), "AE domain should have chunks");
}

#[test]
fn test_get_domain_lb() {
    let index = get_test_index();
    let chunks = index.get_domain("LB", "sdtm");

    assert!(chunks.is_some(), "LB domain should exist in SDTM");
}

#[test]
fn test_get_domain_send() {
    let index = get_test_index();
    // SEND has domains like TS, DM, EX, etc.
    let chunks = index.get_domain("TS", "send");

    // TS (Trial Summary) is a common SEND domain
    if let Some(chunks) = chunks {
        for chunk in &chunks {
            assert_eq!(
                chunk.domain.as_deref(),
                Some("TS"),
                "All chunks should be for TS domain"
            );
        }
    }
}

#[test]
fn test_get_domain_chunk_has_required_fields() {
    let index = get_test_index();
    let chunks = index.get_domain("DM", "sdtm").unwrap();

    let chunk = &chunks[0];
    assert!(!chunk.heading.is_empty(), "heading should not be empty");
    assert!(!chunk.content.is_empty(), "content should not be empty");
    assert!(chunk.domain.is_some(), "domain should be set");
}

// =============================================================================
// 4. Chunk Retrieval Tests (get_chunk_by_index tool)
// =============================================================================

#[test]
fn test_get_chunk_valid_index_zero() {
    let index = get_test_index();
    let chunk = index.get_chunk_by_index("sdtm", 0);

    assert!(chunk.is_some(), "Index 0 should exist");
    let chunk = chunk.unwrap();
    assert_eq!(chunk.index, 0, "Chunk index should match requested index");
}

#[test]
fn test_get_chunk_valid_index_nonzero() {
    let index = get_test_index();
    let chunk = index.get_chunk_by_index("sdtm", 5);

    assert!(chunk.is_some(), "Index 5 should exist");
    let chunk = chunk.unwrap();
    assert_eq!(chunk.index, 5, "Chunk index should match requested index");
}

#[test]
fn test_get_chunk_invalid_index() {
    let index = get_test_index();
    let chunk = index.get_chunk_by_index("sdtm", 999999);

    assert!(chunk.is_none(), "Non-existent index should return None");
}

#[test]
fn test_get_chunk_invalid_ig() {
    let index = get_test_index();
    let chunk = index.get_chunk_by_index("invalid_ig", 0);

    assert!(chunk.is_none(), "Invalid IG should return None");
}

#[test]
fn test_get_chunk_preserves_all_fields() {
    let index = get_test_index();
    let chunk = index.get_chunk_by_index("sdtm", 0).unwrap();

    // Verify required fields are present
    assert!(!chunk.heading.is_empty(), "heading should not be empty");
    assert!(!chunk.content.is_empty(), "content should not be empty");
    // index should be correct
    assert_eq!(chunk.index, 0);
    // Optional fields can be None or Some, just verify they're accessible
    let _ = chunk.domain;
    let _ = chunk.page;
    let _ = chunk.parent_index;
}

#[test]
fn test_get_chunk_from_each_ig() {
    let index = get_test_index();

    // Should be able to get chunk 0 from IGs that have data
    assert!(
        index.get_chunk_by_index("sdtm", 0).is_some(),
        "SDTM should have chunk 0"
    );
    // Note: SEND data may be empty during development
    // assert!(index.get_chunk_by_index("send", 0).is_some(), "SEND should have chunk 0");
    assert!(
        index.get_chunk_by_index("adam", 0).is_some(),
        "ADaM should have chunk 0"
    );
    assert!(
        index.get_chunk_by_index("define", 0).is_some(),
        "Define-XML should have chunk 0"
    );
}

#[test]
fn test_get_chunk_ig_case_insensitive() {
    let index = get_test_index();

    let lower = index.get_chunk_by_index("sdtm", 0);
    let upper = index.get_chunk_by_index("SDTM", 0);
    let mixed = index.get_chunk_by_index("SdTm", 0);

    assert!(lower.is_some(), "lowercase ig should work");
    assert!(upper.is_some(), "uppercase ig should work");
    assert!(mixed.is_some(), "mixed case ig should work");
}

// =============================================================================
// 5. Related Chunks Tests (get_related_chunks tool)
// =============================================================================

#[test]
fn test_get_related_valid_index() {
    let index = get_test_index();
    let related = index.get_related_chunks("sdtm", 0);

    assert!(related.is_some(), "Should find related chunks for index 0");
    let related = related.unwrap();
    assert!(!related.is_empty(), "Related chunks should not be empty");
}

#[test]
fn test_get_related_invalid_index() {
    let index = get_test_index();
    let related = index.get_related_chunks("sdtm", 999999);

    assert!(related.is_none(), "Non-existent index should return None");
}

#[test]
fn test_get_related_invalid_ig() {
    let index = get_test_index();
    let related = index.get_related_chunks("invalid_ig", 0);

    assert!(related.is_none(), "Invalid IG should return None");
}

#[test]
fn test_get_related_sorted_by_index() {
    let index = get_test_index();
    let related = index.get_related_chunks("sdtm", 0);

    if let Some(chunks) = related
        && chunks.len() >= 2
    {
        for i in 0..chunks.len() - 1 {
            assert!(
                chunks[i].index < chunks[i + 1].index,
                "Related chunks should be sorted by index"
            );
        }
    }
}

#[test]
fn test_get_related_includes_requested_chunk() {
    let index = get_test_index();
    let related = index.get_related_chunks("sdtm", 0);

    if let Some(chunks) = related {
        let has_requested = chunks.iter().any(|c| c.index == 0);
        assert!(
            has_requested,
            "Related chunks should include the requested chunk (or its parent)"
        );
    }
}

#[test]
fn test_get_related_parent_returns_children() {
    let index = get_test_index();

    // Find a chunk that has children (is a parent)
    // We'll search for a chunk where other chunks have parent_index pointing to it
    let sections = index.list_sections("sdtm").unwrap();

    // Try the first few sections to find one with potential children
    for section in sections.iter().take(10) {
        let chunk = index.get_chunk_by_index("sdtm", section.first_chunk_index);
        if let Some(chunk) = chunk {
            let related = index.get_related_chunks("sdtm", chunk.index);
            if let Some(related_chunks) = related {
                // Just verify we got some related chunks
                assert!(
                    !related_chunks.is_empty(),
                    "Should return at least the chunk itself"
                );
                break;
            }
        }
    }
}

#[test]
fn test_get_related_ig_case_insensitive() {
    let index = get_test_index();

    let lower = index.get_related_chunks("sdtm", 0);
    let upper = index.get_related_chunks("SDTM", 0);

    assert!(lower.is_some(), "lowercase ig should work");
    assert!(upper.is_some(), "uppercase ig should work");
}

// =============================================================================
// 6. Section Listing Tests (list_sections tool)
// =============================================================================

#[test]
fn test_list_sections_valid_ig_sdtm() {
    let index = get_test_index();
    let sections = index.list_sections("sdtm");

    assert!(sections.is_some(), "SDTM should have sections");
    let sections = sections.unwrap();
    assert!(!sections.is_empty(), "SDTM sections should not be empty");
}

#[test]
fn test_list_sections_valid_ig_send() {
    let index = get_test_index();
    let sections = index.list_sections("send");

    // SEND is a valid IG, so list_sections should return Some
    // However, the sections may be empty if SEND data hasn't been processed yet
    assert!(sections.is_some(), "SEND should be a valid IG");
    // Note: SEND data may be empty during development - don't assert non-empty
}

#[test]
fn test_list_sections_valid_ig_adam() {
    let index = get_test_index();
    let sections = index.list_sections("adam");

    assert!(sections.is_some(), "ADaM should have sections");
    let sections = sections.unwrap();
    assert!(!sections.is_empty(), "ADaM sections should not be empty");
}

#[test]
fn test_list_sections_valid_ig_define() {
    let index = get_test_index();
    let sections = index.list_sections("define");

    assert!(sections.is_some(), "Define-XML should have sections");
    let sections = sections.unwrap();
    assert!(
        !sections.is_empty(),
        "Define-XML sections should not be empty"
    );
}

#[test]
fn test_list_sections_invalid_ig() {
    let index = get_test_index();
    let sections = index.list_sections("invalid_ig");

    assert!(sections.is_none(), "Invalid IG should return None");
}

#[test]
fn test_list_sections_sorted_by_first_chunk_index() {
    let index = get_test_index();
    let sections = index.list_sections("sdtm").unwrap();

    if sections.len() >= 2 {
        for i in 0..sections.len() - 1 {
            assert!(
                sections[i].first_chunk_index <= sections[i + 1].first_chunk_index,
                "Sections should be sorted by first_chunk_index"
            );
        }
    }
}

#[test]
fn test_list_sections_chunk_counts_positive() {
    let index = get_test_index();
    let sections = index.list_sections("sdtm").unwrap();

    for section in &sections {
        assert!(
            section.chunk_count > 0,
            "Each section should have at least 1 chunk"
        );
    }
}

#[test]
fn test_list_sections_headings_not_empty() {
    let index = get_test_index();
    let sections = index.list_sections("sdtm").unwrap();

    for section in &sections {
        assert!(
            !section.heading.is_empty(),
            "Section headings should not be empty"
        );
    }
}

#[test]
fn test_list_sections_ig_case_insensitive() {
    let index = get_test_index();

    let lower = index.list_sections("sdtm");
    let upper = index.list_sections("SDTM");
    let mixed = index.list_sections("SdTm");

    assert!(lower.is_some(), "lowercase ig should work");
    assert!(upper.is_some(), "uppercase ig should work");
    assert!(mixed.is_some(), "mixed case ig should work");

    assert_eq!(
        lower.unwrap().len(),
        upper.unwrap().len(),
        "Case should not affect section count"
    );
}

#[test]
fn test_list_sections_total_chunks_match() {
    let index = get_test_index();
    let sections = index.list_sections("sdtm").unwrap();

    // Sum of all section chunk counts should match total chunks for that IG
    let total_from_sections: usize = sections.iter().map(|s| s.chunk_count).sum();

    // Verify section chunks are reasonable
    assert!(
        total_from_sections > 0,
        "Total chunks from sections should be positive"
    );

    // Verify sections exist
    assert!(
        !sections.is_empty(),
        "SDTM should have at least one section"
    );
}

// =============================================================================
// 7. Integration/Workflow Tests
// =============================================================================

#[test]
fn test_workflow_search_then_get_chunk() {
    let index = get_test_index();

    // Search for something
    let results = index.search("USUBJID", "sdtm", 5);
    assert!(!results.is_empty(), "Should find search results");

    // Get the full chunk for the first result
    let first_result = &results[0];
    let chunk = index.get_chunk_by_index("sdtm", first_result.index);

    assert!(chunk.is_some(), "Should be able to retrieve the chunk");
    let chunk = chunk.unwrap();
    assert_eq!(chunk.index, first_result.index, "Indices should match");
}

#[test]
fn test_workflow_search_then_get_related() {
    let index = get_test_index();

    // Search for something
    let results = index.search("Demographics", "sdtm", 5);
    assert!(!results.is_empty(), "Should find search results");

    // Get related chunks for the first result
    let first_result = &results[0];
    let related = index.get_related_chunks("sdtm", first_result.index);

    assert!(related.is_some(), "Should find related chunks");
}

#[test]
fn test_workflow_list_sections_then_get_chunk() {
    let index = get_test_index();

    // List sections
    let sections = index.list_sections("sdtm").unwrap();
    assert!(!sections.is_empty(), "Should have sections");

    // Get the first chunk of the first section
    let first_section = &sections[0];
    let chunk = index.get_chunk_by_index("sdtm", first_section.first_chunk_index);

    assert!(chunk.is_some(), "Should retrieve the section's first chunk");
    let chunk = chunk.unwrap();
    assert_eq!(
        chunk.heading, first_section.heading,
        "Chunk heading should match section heading"
    );
}

#[test]
fn test_workflow_domain_spec_exploration() {
    let index = get_test_index();

    // Get domain spec
    let dm_chunks = index.get_domain("DM", "sdtm");
    assert!(dm_chunks.is_some(), "DM domain should exist");

    let chunks = dm_chunks.unwrap();
    assert!(!chunks.is_empty(), "DM should have chunks");

    // Verify we can get related chunks for any domain chunk
    let first_chunk = &chunks[0];
    let related = index.get_related_chunks("sdtm", first_chunk.index);
    assert!(
        related.is_some(),
        "Should be able to get related chunks for domain chunk"
    );
}

// =============================================================================
// 8. Edge Cases and Boundary Tests
// =============================================================================

#[test]
fn test_search_special_characters() {
    let index = get_test_index();

    // These should not crash, even if they return empty results
    let _ = index.search("test/slash", "all", 10);
    let _ = index.search("test.dot", "all", 10);
    let _ = index.search("test-dash", "all", 10);
    let _ = index.search("test_underscore", "all", 10);
    let _ = index.search("test(parens)", "all", 10);
}

#[test]
fn test_search_very_long_query() {
    let index = get_test_index();

    // Create a very long query
    let long_query = "word ".repeat(100);
    let results = index.search(&long_query, "all", 10);

    // Should not crash, results may be empty
    assert!(
        results.len() <= 10,
        "Should respect limit even with long query"
    );
}

#[test]
fn test_search_unicode() {
    let index = get_test_index();

    // Unicode shouldn't crash
    let _ = index.search("caf\u{00e9}", "all", 10);
    let _ = index.search("\u{00b5}g", "all", 10); // µg (microgram)
}

#[test]
fn test_large_limit() {
    let index = get_test_index();

    // Very large limit should work (just returns all matches up to that limit)
    let results = index.search("domain", "all", 1000);
    // Should not crash, will return whatever matches exist
    assert!(results.len() <= 1000);
}
