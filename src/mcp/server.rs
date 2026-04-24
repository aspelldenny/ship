use std::future::Future;
use std::sync::Arc;

use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Implementation, ListToolsResult, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::service::RoleServer;
use rmcp::{tool, tool_router};

use crate::config::Config;
use crate::mcp::tools::{
    CanaryParams, CheckParams, LearnAddParams, LearnSearchParams, NoteExportParams,
};

#[allow(dead_code)]
pub struct ShipServer {
    config: Arc<Config>,
    tool_router: ToolRouter<Self>,
}

impl ShipServer {
    pub fn new(config: Config) -> Self {
        let tool_router = Self::tool_router();
        Self {
            config: Arc::new(config),
            tool_router,
        }
    }
}

#[tool_router]
impl ShipServer {
    /// Run pre-flight checks (test + docs-gate, no commit)
    #[tool(name = "ship_check")]
    fn ship_check(&self, Parameters(params): Parameters<CheckParams>) -> String {
        let config = &self.config;
        let opts = crate::pipeline::PipelineOptions {
            dry_run: true,
            skip_tests: params.skip_tests.unwrap_or(false),
            skip_docs_gate: params.skip_docs_gate.unwrap_or(false),
            bump: None,
            no_pr: true,
            verbose: false,
        };

        match crate::pipeline::check(config, &opts) {
            Ok(result) => {
                let steps: Vec<String> = result
                    .steps
                    .iter()
                    .map(|s| {
                        let icon = match &s.status {
                            crate::pipeline::StepStatus::Pass => "PASS",
                            crate::pipeline::StepStatus::Fail(_) => "FAIL",
                            crate::pipeline::StepStatus::Warn(_) => "WARN",
                            crate::pipeline::StepStatus::Skip(_) => "SKIP",
                        };
                        let detail = match &s.status {
                            crate::pipeline::StepStatus::Pass => {
                                s.output.as_deref().unwrap_or("ok")
                            }
                            crate::pipeline::StepStatus::Fail(m) => m.as_str(),
                            crate::pipeline::StepStatus::Warn(m) => m.as_str(),
                            crate::pipeline::StepStatus::Skip(m) => m.as_str(),
                        };
                        format!("[{icon}] {}: {detail}", s.name)
                    })
                    .collect();
                steps.join("\n")
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Run health check on deployed application
    #[tool(name = "ship_canary")]
    async fn ship_canary(&self, Parameters(params): Parameters<CanaryParams>) -> String {
        let mut canary_config = self.config.canary.clone();
        if let Some(url) = params.url {
            canary_config.url = Some(url);
        }
        if let Some(timeout) = params.timeout_secs {
            canary_config.timeout_secs = timeout;
        }

        match crate::canary::run(&canary_config).await {
            Ok(result) => {
                let checks: Vec<String> = result
                    .checks
                    .iter()
                    .map(|c| {
                        let status = match &c.status {
                            crate::canary::CanaryStatus::Healthy => "HEALTHY",
                            crate::canary::CanaryStatus::Degraded(_) => "DEGRADED",
                            crate::canary::CanaryStatus::Down(_) => "DOWN",
                        };
                        let latency = c
                            .latency_ms
                            .map(|ms| format!(" ({ms}ms)"))
                            .unwrap_or_default();
                        format!("[{status}] {}{latency}", c.name)
                    })
                    .collect();
                checks.join("\n")
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Record a project learning
    #[tool(name = "ship_learn_add")]
    fn ship_learn_add(&self, Parameters(params): Parameters<LearnAddParams>) -> String {
        let project = self.config.project_name();
        match crate::learn::add(&self.config.learn, &project, &params.message, &params.tags) {
            Ok(_) => format!("Saved: {}", params.message),
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Search project learnings by keyword
    #[tool(name = "ship_learn_search")]
    fn ship_learn_search(&self, Parameters(params): Parameters<LearnSearchParams>) -> String {
        let project = self.config.project_name();
        let path = crate::learn::resolve_path_pub(&self.config.learn, &project);
        match crate::learn::store::search(&path, &params.query) {
            Ok(results) => {
                if results.is_empty() {
                    format!("No learnings found for \"{}\"", params.query)
                } else {
                    results
                        .iter()
                        .map(|l| {
                            let tags = if l.tags.is_empty() {
                                String::new()
                            } else {
                                format!(" [{}]", l.tags.join(", "))
                            };
                            format!("• {}{} ({})", l.message, tags, l.project)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Export a ship note to the Obsidian vault (per-phiếu log)
    #[tool(name = "ship_note_export")]
    fn ship_note_export(&self, Parameters(params): Parameters<NoteExportParams>) -> String {
        let opts = crate::note::NoteOptions {
            project: params.project_slug,
            ticket: params.ticket_id,
            message: params.message,
            vault_path: params.vault_path,
        };
        match crate::note::run(&self.config.obsidian, opts) {
            crate::note::NoteOutcome::Written(p) => format!("Written: {}", p.display()),
            crate::note::NoteOutcome::Skipped(reason) => format!("Skipped: {reason}"),
        }
    }
}

impl rmcp::ServerHandler for ShipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(String::from(
                "Ship CLI tools for automated release workflow. \
                 ship_check: run pre-flight checks (test + docs-gate). \
                 ship_canary: health check deployed app. \
                 ship_learn_add: record a project learning. \
                 ship_learn_search: search learnings by keyword. \
                 ship_note_export: write a per-phiếu ship note into the Obsidian vault.",
            )),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
        let tools = self.tool_router.list_all();
        std::future::ready(Ok(ListToolsResult {
            tools,
            next_cursor: None,
        }))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
        let tool_context = ToolCallContext::new(self, request, context);
        async move {
            self.tool_router
                .call(tool_context)
                .await
                .map_err(|e| rmcp::ErrorData::new(e.code, e.message, e.data))
        }
    }
}

pub async fn serve(config: Config) -> crate::error::Result<()> {
    use rmcp::ServiceExt;

    let server = ShipServer::new(config);
    let transport = rmcp::transport::io::stdio();
    let service = server
        .serve(transport)
        .await
        .map_err(|e| crate::error::ShipError::Config(format!("MCP server error: {e}")))?;
    service
        .waiting()
        .await
        .map_err(|e| crate::error::ShipError::Config(format!("MCP server error: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info() {
        let server = ShipServer::new(Config::default());
        let info = rmcp::ServerHandler::get_info(&server);
        assert_eq!(info.server_info.name, "ship");
    }

    #[test]
    fn test_tool_router_has_5_tools() {
        let server = ShipServer::new(Config::default());
        let tools = server.tool_router.list_all();
        assert_eq!(tools.len(), 5);
        let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
        assert!(names.contains(&"ship_check".to_string()));
        assert!(names.contains(&"ship_canary".to_string()));
        assert!(names.contains(&"ship_learn_add".to_string()));
        assert!(names.contains(&"ship_learn_search".to_string()));
        assert!(names.contains(&"ship_note_export".to_string()));
    }
}
