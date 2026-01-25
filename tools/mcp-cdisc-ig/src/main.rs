//! CDISC Implementation Guide MCP Server
//!
//! Provides tools to search and query CDISC Implementation Guides:
//! - SDTM-IG v3.4 (461 pages)
//! - SEND-IG v3.1.1 (244 pages)
//! - ADaM-IG v1.3 (88 pages)

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

mod index;

// Parameter structs for tools

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchIgParams {
    /// Search query (e.g., 'USUBJID derivation', 'DM domain requirements')
    query: String,
    /// Which IG to search: 'sdtm', 'send', 'adam', or 'all' (default: 'all')
    ig: Option<String>,
    /// Maximum results to return (default: 10, max: 50)
    limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetDomainSpecParams {
    /// Domain code (e.g., 'DM', 'AE', 'LB', 'EX', 'VS', 'CM')
    domain: String,
    /// Implementation Guide: 'sdtm', 'send', or 'adam'
    ig: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetVariableSpecParams {
    /// Variable name (e.g., 'USUBJID', 'AETERM', 'LBTEST', 'EXDOSE')
    variable: String,
    /// Domain code to narrow search (optional, e.g., 'DM', 'AE')
    domain: Option<String>,
}

/// CDISC Implementation Guide MCP Server
#[derive(Clone)]
pub struct CdiscIgServer {
    index: Arc<RwLock<index::IgIndex>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CdiscIgServer {
    pub fn new(index: index::IgIndex) -> Self {
        Self {
            index: Arc::new(RwLock::new(index)),
            tool_router: Self::tool_router(),
        }
    }

    /// Search across CDISC Implementation Guides (SDTM, SEND, ADaM)
    #[tool(
        description = "Search CDISC Implementation Guides for specific topics. Returns relevant sections from SDTM-IG v3.4, SEND-IG v3.1.1, or ADaM-IG v1.3 with page numbers and context."
    )]
    async fn search_ig(
        &self,
        params: Parameters<SearchIgParams>,
    ) -> Result<CallToolResult, McpError> {
        let SearchIgParams { query, ig, limit } = params.0;
        let index = self.index.read().await;
        let ig = ig.unwrap_or_else(|| "all".to_string());
        let limit = limit.unwrap_or(10).min(50);

        let results = index.search(&query, &ig, limit);

        if results.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No results found for '{}' in {} IG(s)",
                query, ig
            ))]));
        }

        let content = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    /// Get guidance text related to a CDISC domain
    #[tool(
        description = "Get all guidance text chunks related to a specific CDISC domain. Returns IG sections that discuss the domain's purpose, structure, and compliance requirements."
    )]
    async fn get_domain_spec(
        &self,
        params: Parameters<GetDomainSpecParams>,
    ) -> Result<CallToolResult, McpError> {
        let GetDomainSpecParams { domain, ig } = params.0;
        let ig_lower = ig.to_lowercase();
        if !["sdtm", "send", "adam"].contains(&ig_lower.as_str()) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Invalid IG '{}'. Must be one of: sdtm, send, adam",
                ig
            ))]));
        }

        let index = self.index.read().await;

        match index.get_domain(&domain, &ig_lower) {
            Some(spec) => {
                let content = serde_json::to_string_pretty(&spec)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![Content::text(content)]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "Domain '{}' not found in {}-IG. Use search_ig to find available domains.",
                domain.to_uppercase(),
                ig.to_uppercase()
            ))])),
        }
    }

    /// Get guidance text related to a CDISC variable
    #[tool(
        description = "Get all guidance text chunks that discuss a specific CDISC variable. Returns IG sections explaining the variable's derivation, usage rules, and compliance requirements."
    )]
    async fn get_variable_spec(
        &self,
        params: Parameters<GetVariableSpecParams>,
    ) -> Result<CallToolResult, McpError> {
        let GetVariableSpecParams { variable, domain } = params.0;
        let index = self.index.read().await;

        let specs = index.get_variable(&variable, domain.as_deref());
        if specs.is_empty() {
            let msg = match domain {
                Some(d) => format!(
                    "Variable '{}' not found in domain '{}'. \
                     Try without specifying a domain to search all IGs.",
                    variable.to_uppercase(),
                    d.to_uppercase()
                ),
                None => format!(
                    "Variable '{}' not found in any IG. \
                     Check the variable name or use search_ig to find similar variables.",
                    variable.to_uppercase()
                ),
            };
            Ok(CallToolResult::success(vec![Content::text(msg)]))
        } else {
            let content = serde_json::to_string_pretty(&specs)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
            Ok(CallToolResult::success(vec![Content::text(content)]))
        }
    }
}

#[tool_handler]
impl ServerHandler for CdiscIgServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "cdisc-ig".to_string(),
                title: Some("CDISC Implementation Guide Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "CDISC Implementation Guide documentation server. \
                 Provides searchable access to SDTM-IG v3.4, SEND-IG v3.1.1, \
                 and ADaM-IG v1.3 specifications."
                    .to_string(),
            ),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (MCP spec: servers MAY use stderr for logging)
    tracing_subscriber::fmt()
        .with_env_filter("mcp_cdisc_ig=info")
        .with_writer(std::io::stderr)
        .init();

    tracing::info!(
        "Starting CDISC-IG MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load pre-indexed IG content
    let index = index::IgIndex::load()?;
    tracing::info!(
        "Loaded {} sections from {} domains across 3 IGs",
        index.section_count(),
        index.domain_count()
    );

    // Create the MCP server
    let server = CdiscIgServer::new(index);

    // Start server on stdio transport (MCP standard transport)
    let transport = stdio();
    let running = server.serve(transport).await?;

    tracing::info!("Server ready, waiting for requests on stdio...");

    // Wait for shutdown
    running.waiting().await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
