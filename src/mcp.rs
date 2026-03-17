use crate::checker;
use crate::store;
use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::*, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckAppParams {
    #[schemars(description = "App name to check")]
    pub name: String,
    #[schemars(
        description = "Optional comma-separated store IDs to check (defaults to all stores: app_store, google_play)"
    )]
    pub stores: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckAppsParams {
    #[schemars(description = "List of app names to check")]
    pub names: Vec<String>,
    #[schemars(
        description = "Optional comma-separated store IDs to check (defaults to all stores)"
    )]
    pub stores: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListStoresParams {}

pub struct PublishedMcp {
    tool_router: ToolRouter<Self>,
}

impl Default for PublishedMcp {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_stores(stores: &Option<String>) -> Vec<store::Store> {
    match stores {
        Some(ids) => {
            let ids: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
            store::stores_by_ids(&ids)
        }
        None => store::all_stores().to_vec(),
    }
}

#[tool_router]
impl PublishedMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[rmcp::tool(
        description = "Check if an app name is available on app stores (Apple App Store, Google Play). Returns availability status for each store checked."
    )]
    async fn check_app(
        &self,
        Parameters(params): Parameters<CheckAppParams>,
    ) -> Result<CallToolResult, McpError> {
        let stores = resolve_stores(&params.stores);
        if stores.is_empty() {
            return Err(McpError::invalid_params("No valid stores specified", None));
        }
        let result = checker::check_app(&params.name, &stores).await;
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(
        description = "Check multiple app names for availability across app stores. Runs all checks concurrently."
    )]
    async fn check_apps(
        &self,
        Parameters(params): Parameters<CheckAppsParams>,
    ) -> Result<CallToolResult, McpError> {
        if params.names.is_empty() {
            return Err(McpError::invalid_params("names list cannot be empty", None));
        }
        if params.names.len() > 50 {
            return Err(McpError::invalid_params(
                "Maximum 50 names per request",
                None,
            ));
        }
        let stores = resolve_stores(&params.stores);
        if stores.is_empty() {
            return Err(McpError::invalid_params("No valid stores specified", None));
        }
        let results = checker::check_apps(&params.names, &stores).await;
        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[rmcp::tool(
        description = "List all available app stores with their IDs, names, and platforms."
    )]
    async fn list_stores(
        &self,
        Parameters(_params): Parameters<ListStoresParams>,
    ) -> Result<CallToolResult, McpError> {
        let infos: Vec<_> = store::all_stores().iter().map(|s| s.info()).collect();
        let json = serde_json::to_string_pretty(&infos)
            .map_err(|e| McpError::internal_error(format!("Serialization error: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[tool_handler]
impl ServerHandler for PublishedMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "App store name availability checker. Use check_app for a single name, \
                 check_apps for bulk lookups, or list_stores to see available stores."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            ..Default::default()
        }
    }
}
