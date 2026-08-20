use crate::tool_trait::{ToolError, ToolExecutionContext, ToolResult, XenoTool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// MCP tool schema representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolSchema {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Dynamic registry holding tools accessible by agents and external MCP clients.
#[derive(Default, Clone)]
pub struct McpToolRegistry {
    tools: HashMap<String, Arc<dyn XenoTool>>,
}

impl McpToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a new tool in the registry.
    pub fn register_tool(&mut self, tool: Arc<dyn XenoTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Retrieves an MCP tool definition by name.
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn XenoTool>> {
        self.tools.get(name).cloned()
    }

    /// Lists all registered tools as MCP tool schemas.
    pub fn list_mcp_tools(&self) -> Vec<McpToolSchema> {
        self.tools
            .values()
            .map(|t| {
                let def = t.definition();
                McpToolSchema {
                    name: def.name,
                    description: def.description,
                    input_schema: def.parameters,
                }
            })
            .collect()
    }

    /// Executes a registered tool by name with arguments and execution context.
    pub async fn execute_tool(
        &self,
        name: &str,
        args: Value,
        ctx: &ToolExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let tool = self.tools.get(name).ok_or_else(|| ToolError::InvalidArguments(
            format!("Tool '{name}' not found in registry"),
        ))?;

        if tool.security_tier() > ctx.current_tier_approval {
            return Err(ToolError::PermissionDenied {
                required: tool.security_tier(),
                current: ctx.current_tier_approval,
                reason: format!("Tool '{name}' requires {:?} authorization", tool.security_tier()),
            });
        }

        tool.execute(args, ctx).await
    }
}
