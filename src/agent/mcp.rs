// SPDX-License-Identifier: GPL-3.0-or-later

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt, model::*,
    service::RequestContext, transport::stdio,
};
use serde_json::json;
use std::sync::Arc;

use crate::backends::DnsBackend;
use crate::error::AppError;
use crate::output::mode::{ColorMode, Format, OutputMode};
use crate::vpn::VpnProvider;

/// Launch MCP server with stdio transport.
pub async fn run_mcp_server() -> Result<(), AppError> {
    let server = FluxMcpServer::new();
    let transport = stdio();
    let running = server
        .serve(transport)
        .await
        .map_err(|e| AppError::general(format!("MCP server initialization failed: {e}")))?;
    running
        .waiting()
        .await
        .map_err(|e| AppError::general(format!("MCP server error: {e}")))?;
    Ok(())
}

#[derive(Clone)]
pub struct FluxMcpServer;

impl FluxMcpServer {
    pub fn new() -> Self {
        Self
    }

    /// Build the JSON output mode used for all MCP tool executions.
    fn json_mode() -> OutputMode {
        OutputMode {
            format: Format::Json,
            color: ColorMode::Never,
            interactive: false,
            verbose: false,
            quiet: true,
            dry_run: false,
            yes: true,
            fields: None,
            absolute_time: true,
            print0: false,
        }
    }

    // ------------------------------------------------------------------
    // Full tool definitions (for validation & tools_get)
    // ------------------------------------------------------------------
    fn full_tools() -> Vec<Tool> {
        vec![
            Self::tool_apply(),
            Self::tool_status(),
            Self::tool_list(),
            Self::tool_detect(),
            Self::tool_verify(),
            Self::tool_tools_get(),
        ]
    }

    fn tool_apply() -> Tool {
        let schema = json!({
            "type": "object",
            "properties": {
                "provider": { "type": "string", "description": "Provider slug, e.g. cloudflare" },
                "tier": { "type": "string", "description": "Filtering tier, e.g. family" },
                "protocol": { "type": "string", "description": "Transport protocol, e.g. doh" },
                "ipv4_only": { "type": "boolean" },
                "ipv6_only": { "type": "boolean" },
                "no_backup": { "type": "boolean" },
                "no_verify": { "type": "boolean" }
            },
            "required": ["provider", "protocol"]
        });
        Tool::new(
            "apply",
            "Apply DNS configuration [write]",
            rmcp::model::object(schema),
        )
        .with_annotations(ToolAnnotations::new().read_only(false).destructive(false))
    }

    fn tool_status() -> Tool {
        Tool::new(
            "status",
            "Show DNS/NTP/VPN state [read]",
            Self::schema_empty_object(),
        )
        .with_annotations(ToolAnnotations::new().read_only(true))
    }

    fn tool_list() -> Tool {
        let schema = json!({
            "type": "object",
            "properties": {
                "providers": { "type": "boolean" },
                "tiers": { "type": "boolean" },
                "protocols": { "type": "boolean" },
                "vpn": { "type": "boolean" },
                "provider": { "type": "string" },
                "tier": { "type": "string" }
            }
        });
        Tool::new(
            "list",
            "List providers/tiers/protocols [read]",
            rmcp::model::object(schema),
        )
        .with_annotations(ToolAnnotations::new().read_only(true))
    }

    fn tool_detect() -> Tool {
        Tool::new(
            "detect",
            "Detect active backend [read]",
            Self::schema_empty_object(),
        )
        .with_annotations(ToolAnnotations::new().read_only(true))
    }

    fn tool_verify() -> Tool {
        Tool::new(
            "verify",
            "Test DNS resolution [read]",
            Self::schema_empty_object(),
        )
        .with_annotations(ToolAnnotations::new().read_only(true))
    }

    fn tool_tools_get() -> Tool {
        let schema = json!({
            "type": "object",
            "properties": {
                "tool_name": { "type": "string", "description": "Name of the tool to retrieve schema for" }
            },
            "required": ["tool_name"]
        });
        Tool::new(
            "tools_get",
            "Return full JSON Schema for a named tool [read]",
            rmcp::model::object(schema),
        )
        .with_annotations(ToolAnnotations::new().read_only(true))
    }

    fn schema_empty_object() -> Arc<JsonObject> {
        Arc::new(rmcp::model::object(
            json!({"type": "object", "properties": {}}),
        ))
    }

    // ------------------------------------------------------------------
    // Lazy-loading: stripped tool list (no heavy schemas)
    // ------------------------------------------------------------------
    fn stripped_tools() -> Vec<Tool> {
        Self::full_tools()
            .into_iter()
            .map(|t| {
                Tool::new(
                    t.name,
                    t.description.unwrap_or_default(),
                    Self::schema_empty_object(),
                )
                .with_annotations(t.annotations.unwrap_or_default())
            })
            .collect()
    }

    fn full_schema_for_tool(tool_name: &str) -> Option<serde_json::Value> {
        Self::full_tools()
            .into_iter()
            .find(|t| t.name == tool_name)
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema,
                    "outputSchema": t.output_schema,
                    "annotations": t.annotations,
                })
            })
    }

    // ------------------------------------------------------------------
    // Tool dispatch
    // ------------------------------------------------------------------
    fn dispatch_tool(
        &self,
        name: &str,
        arguments: Option<JsonObject>,
    ) -> Result<CallToolResult, McpError> {
        let args = arguments.unwrap_or_default();
        match name {
            "apply" => self.handle_apply(&args),
            "status" => self.handle_status(),
            "list" => self.handle_list(&args),
            "detect" => self.handle_detect(),
            "verify" => self.handle_verify(),
            "tools_get" => self.handle_tools_get(&args),
            _ => Err(McpError::invalid_params(
                format!("Unknown tool: {name}"),
                None,
            )),
        }
    }

    fn handle_apply(&self, args: &JsonObject) -> Result<CallToolResult, McpError> {
        let provider = args
            .get("provider")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing 'provider' argument", None))?;
        let protocol_str = args
            .get("protocol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing 'protocol' argument", None))?;

        crate::validate::reject_control_chars(provider)
            .map_err(|e| McpError::invalid_params(e.message, None))?;
        crate::validate::reject_control_chars(protocol_str)
            .map_err(|e| McpError::invalid_params(e.message, None))?;

        let tier = args
            .get("tier")
            .and_then(|v| v.as_str())
            .map(|t| {
                crate::validate::reject_control_chars(t)
                    .map_err(|e| McpError::invalid_params(e.message, None))?;
                crate::validate::reject_prompt_injection(t)
                    .map_err(|e| McpError::invalid_params(e.message, None))?;
                t.parse::<crate::registry::Tier>()
                    .map_err(|e| McpError::invalid_params(e.message, None))
            })
            .transpose()?;

        let protocol = protocol_str
            .parse::<crate::registry::Protocol>()
            .map_err(|e| McpError::invalid_params(e.message, None))?;

        let ipv4_only = args
            .get("ipv4_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let ipv6_only = args
            .get("ipv6_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let no_backup = args
            .get("no_backup")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let no_verify = args
            .get("no_verify")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mode = Self::json_mode();
        let result = crate::orchestrator::run_apply(
            provider, tier, protocol, ipv4_only, ipv6_only, no_backup, no_verify, false, None,
            &mode,
        )
        .map_err(|e| {
            McpError::internal_error(
                e.message,
                Some(json!({"code": e.code, "hint": e.hint, "exit_code": e.exit_code})),
            )
        })?;

        let text = serde_json::to_string(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    fn handle_status(&self) -> Result<CallToolResult, McpError> {
        let backend = crate::detection::detect_backend().map_err(|e| {
            McpError::internal_error(e.message, Some(json!({"code": e.code, "hint": e.hint})))
        })?;

        let dns_status = match backend {
            crate::detection::Backend::ResolvConf => {
                crate::backends::resolvconf::ResolvConfBackend.status()
            }
            crate::detection::Backend::SystemdResolved => {
                crate::backends::systemd::SystemdResolvedBackend.status()
            }
            crate::detection::Backend::NetworkManager => {
                crate::backends::networkmanager::NetworkManagerBackend.status()
            }
            _ => Ok(crate::backends::BackendStatus {
                backend: backend.to_string(),
                active: true,
                nameservers: None,
                dot_enabled: None,
                doh_enabled: None,
            }),
        }
        .map_err(|e| {
            McpError::internal_error(e.message, Some(json!({"code": e.code, "hint": e.hint})))
        })?;

        let status = json!({
            "backend": backend.to_string(),
            "dns": dns_status,
        });
        let text = serde_json::to_string(&status)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    fn handle_list(&self, args: &JsonObject) -> Result<CallToolResult, McpError> {
        let providers = args
            .get("providers")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let tiers = args.get("tiers").and_then(|v| v.as_bool()).unwrap_or(false);
        let protocols = args
            .get("protocols")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let vpn = args.get("vpn").and_then(|v| v.as_bool()).unwrap_or(false);
        let provider = args.get("provider").and_then(|v| v.as_str());
        let tier = args.get("tier").and_then(|v| v.as_str());

        let data = if providers {
            let providers: Vec<_> = crate::registry::list_providers()
                .into_iter()
                .map(|p| {
                    json!({
                        "slug": p.slug,
                        "name": p.name,
                        "tiers": p.tiers.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                        "protocols": p.protocols.iter().map(|pr| pr.to_string()).collect::<Vec<_>>(),
                        "ntp_server": p.ntp_server,
                    })
                })
                .collect();
            json!({ "providers": providers })
        } else if tiers {
            let slug = provider.unwrap_or("cloudflare");
            let p = crate::registry::get_provider(slug).ok_or_else(|| {
                McpError::invalid_params(format!("Unknown provider: {slug}"), None)
            })?;
            let tiers: Vec<_> = p.tiers.iter().map(|t| t.to_string()).collect();
            json!({ "provider": slug, "tiers": tiers })
        } else if protocols {
            let slug = provider.unwrap_or("cloudflare");
            let p = crate::registry::get_provider(slug).ok_or_else(|| {
                McpError::invalid_params(format!("Unknown provider: {slug}"), None)
            })?;
            let t = tier
                .map(|t| t.parse::<crate::registry::Tier>())
                .transpose()
                .map_err(|e| McpError::invalid_params(e.message, None))?;
            let backend = crate::detection::detect_backend().map_err(|e| {
                McpError::internal_error(e.message, Some(json!({"code": e.code, "hint": e.hint})))
            })?;
            let protocols: Vec<_> = crate::registry::valid_protocols(p, t, backend)
                .iter()
                .map(|pr| pr.to_string())
                .collect();
            json!({ "provider": slug, "tier": t.map(|v| v.to_string()), "protocols": protocols })
        } else if vpn {
            let warp = crate::vpn::warp::WarpProvider;
            let adguard = crate::vpn::adguard::AdGuardVpnProvider;
            json!({
                "vpn_clients": [
                    { "name": "warp", "available": warp.is_available() },
                    { "name": "adguard", "available": adguard.is_available() }
                ]
            })
        } else {
            json!({
                "providers": crate::registry::list_providers().len(),
                "message": "Use providers, tiers, protocols, or vpn flags"
            })
        };

        let text = serde_json::to_string(&data)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    fn handle_detect(&self) -> Result<CallToolResult, McpError> {
        let backend = crate::detection::detect_backend().map_err(|e| {
            McpError::internal_error(e.message, Some(json!({"code": e.code, "hint": e.hint})))
        })?;
        let ntp = crate::detection::detect_ntp_backend().ok();
        let pkg_mgr = crate::detection::detect_package_manager();
        let data = json!({
            "dns_backend": backend.to_string(),
            "ntp_backend": ntp.map(|b| format!("{b:?}")),
            "package_manager": pkg_mgr.map(|p| format!("{p:?}")),
        });
        let text = serde_json::to_string(&data)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    fn handle_verify(&self) -> Result<CallToolResult, McpError> {
        let backend = crate::detection::detect_backend().map_err(|e| {
            McpError::internal_error(e.message, Some(json!({"code": e.code, "hint": e.hint})))
        })?;
        let result = match backend {
            crate::detection::Backend::ResolvConf => {
                crate::backends::resolvconf::ResolvConfBackend.verify(5)
            }
            crate::detection::Backend::SystemdResolved => {
                crate::backends::systemd::SystemdResolvedBackend.verify(5)
            }
            crate::detection::Backend::NetworkManager => {
                crate::backends::networkmanager::NetworkManagerBackend.verify(5)
            }
            _ => {
                return Err(McpError::internal_error(
                    format!("Verify not implemented for backend: {backend}"),
                    None,
                ));
            }
        }
        .map_err(|e| {
            McpError::internal_error(e.message, Some(json!({"code": e.code, "hint": e.hint})))
        })?;
        let value = serde_json::to_value(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let text = serde_json::to_string(&value)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    fn handle_tools_get(&self, args: &JsonObject) -> Result<CallToolResult, McpError> {
        let tool_name = args
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing 'tool_name' argument", None))?;

        match Self::full_schema_for_tool(tool_name) {
            Some(schema) => {
                let text = serde_json::to_string(&schema)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            None => Err(McpError::invalid_params(
                format!("Unknown tool: {tool_name}"),
                None,
            )),
        }
    }
}

impl Default for FluxMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for FluxMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("flux-mcp", env!("CARGO_PKG_VERSION"))
                    .with_title("Flux MCP Server")
                    .with_description("DNS selector and network configurator MCP server")
                    .with_website_url("https://Flux.Steelbore.com/"),
            )
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "Flux MCP server provides DNS configuration tools. \
             Use tools_get to retrieve full JSON schemas for any tool."
                    .to_string(),
            )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: Self::stripped_tools(),
            next_cursor: None,
            meta: None,
        }))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        Self::full_tools().into_iter().find(|t| t.name == name)
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        let name = request.name.to_string();
        let arguments = request.arguments;
        async move { self.dispatch_tool(&name, arguments) }
    }
}
