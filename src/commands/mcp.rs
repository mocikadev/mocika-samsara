use crate::{
    cli::{McpAction, McpArgs},
    config::Config,
    error::SamsaraError,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    ffi::OsString,
    fs,
    io::{self, BufRead, Write},
    process::Command,
};

pub fn run(args: McpArgs, config: &Config) -> Result<(), SamsaraError> {
    match args.action {
        McpAction::Serve { port: _ } => serve_stdio(config),
    }
}

fn serve_stdio(config: &Config) -> Result<(), SamsaraError> {
    eprintln!("samsara MCP server started (stdio)");

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if let Some(response) = handle_request(&line, config) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }

    Ok(())
}

fn handle_request(line: &str, config: &Config) -> Option<Value> {
    match serde_json::from_str::<JsonRpcRequest>(line) {
        Ok(request) => {
            // JSON-RPC 2.0 notification: id absent → id is None → no response
            let id = request.id.clone()?;
            Some(dispatch_request_with_id(id, request, config))
        }
        Err(error) => Some(json!({
            "jsonrpc": "2.0",
            "id": Value::Null,
            "error": {
                "code": -32700,
                "message": format!("invalid json: {error}"),
            }
        })),
    }
}

fn dispatch_request_with_id(id: Value, request: JsonRpcRequest, config: &Config) -> Value {
    if request.jsonrpc != "2.0" {
        return error_response(
            id,
            -32600,
            format!("invalid jsonrpc version: {}", request.jsonrpc),
        );
    }

    match request.method.as_str() {
        "initialize" => success_response(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "samsara",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {
                    "tools": {
                        "listChanged": false,
                    }
                }
            }),
        ),
        "tools/list" => success_response(id, json!({ "tools": tool_definitions() })),
        "tools/call" => match parse_tool_call(request.params) {
            Ok(tool_call) => match call_tool(&tool_call.name, &tool_call.arguments, config) {
                Ok(result) => success_response(id, result),
                Err(error) => error_response(id, -32001, error.to_string()),
            },
            Err(error) => error_response(id, -32602, error.to_string()),
        },
        _ => error_response(id, -32601, format!("method not found: {}", request.method)),
    }
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "write_lesson",
            "description": "写入或更新 lesson",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string" },
                    "keyword": { "type": "string" },
                    "summary": { "type": "string" }
                },
                "required": ["domain", "keyword", "summary"]
            }
        }),
        json!({
            "name": "search_knowledge",
            "description": "搜索知识库",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "get_status",
            "description": "获取知识库状态",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "promote_lesson",
            "description": "晋升 lesson",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string" },
                    "keyword": { "type": "string" },
                    "target": { "type": "string", "enum": ["rules", "layer0"] }
                },
                "required": ["domain", "keyword", "target"]
            }
        }),
        json!({
            "name": "read_index",
            "description": "读取 INDEX.md",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "prime_context",
            "description": "生成紧凑上下文摘要",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domains": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "limit": { "type": "integer" }
                }
            }
        }),
    ]
}

fn call_tool(name: &str, arguments: &Value, config: &Config) -> Result<Value, SamsaraError> {
    match name {
        "write_lesson" => {
            let domain = required_string(arguments, "domain")?;
            let keyword = required_string(arguments, "keyword")?;
            let summary = required_string(arguments, "summary")?;
            let output = run_cli_capture(
                config,
                [
                    OsString::from("write"),
                    OsString::from(domain),
                    OsString::from(keyword),
                    OsString::from("--summary"),
                    OsString::from(summary),
                    OsString::from("--yes"),
                ],
            )?;
            Ok(command_result(output))
        }
        "search_knowledge" => {
            let query = required_string(arguments, "query")?;
            let limit = optional_u64(arguments, "limit").unwrap_or(10).to_string();
            let output = run_cli_capture(
                config,
                [
                    OsString::from("search"),
                    OsString::from(query),
                    OsString::from("--limit"),
                    OsString::from(limit),
                ],
            )?;
            Ok(command_result(output))
        }
        "get_status" => {
            let output = run_cli_capture(config, [OsString::from("status")])?;
            Ok(command_result(output))
        }
        "promote_lesson" => {
            let domain = required_string(arguments, "domain")?;
            let keyword = required_string(arguments, "keyword")?;
            let target = required_string(arguments, "target")?;
            let mut args = vec![
                OsString::from("promote"),
                OsString::from(domain),
                OsString::from(keyword),
            ];
            if target == "layer0" {
                args.push(OsString::from("--layer0"));
                args.push(OsString::from("--yes"));
            }
            let output = run_cli_capture(config, args)?;
            Ok(command_result(output))
        }
        "read_index" => {
            let content = fs::read_to_string(config.knowledge_home.join("INDEX.md"))?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": content,
                }],
                "structuredContent": {
                    "path": config.knowledge_home.join("INDEX.md").display().to_string(),
                }
            }))
        }
        "prime_context" => {
            let limit = optional_u64(arguments, "limit").unwrap_or(10);
            let domains = optional_string_array(arguments, "domains");
            let mut combined = Vec::new();

            if domains.is_empty() {
                combined.push(run_cli_capture(
                    config,
                    [
                        OsString::from("prime"),
                        OsString::from("--limit"),
                        OsString::from(limit.to_string()),
                    ],
                )?);
            } else {
                for domain in domains {
                    combined.push(run_cli_capture(
                        config,
                        [
                            OsString::from("prime"),
                            OsString::from("--limit"),
                            OsString::from(limit.to_string()),
                            OsString::from("--domain"),
                            OsString::from(domain),
                        ],
                    )?);
                }
            }

            let text = combined
                .into_iter()
                .map(output_text)
                .filter(|item| !item.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text,
                }],
                "structuredContent": {
                    "limit": limit,
                }
            }))
        }
        _ => Err(SamsaraError::UpdateError(format!("未知工具：{name}"))),
    }
}

fn run_cli_capture<I>(config: &Config, args: I) -> Result<std::process::Output, SamsaraError>
where
    I: IntoIterator<Item = OsString>,
{
    let exe_path = std::env::current_exe()?;
    let mut command = Command::new(exe_path);
    command.arg("--home").arg(&config.knowledge_home);
    command.args(args);
    command.output().map_err(SamsaraError::from)
}

fn command_result(output: std::process::Output) -> Value {
    let is_error = !output.status.success();
    let text = output_text(output);
    json!({
        "content": [{
            "type": "text",
            "text": text,
        }],
        "isError": is_error,
    })
}

fn output_text(output: std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    match (
        stdout.is_empty(),
        stderr.is_empty(),
        output.status.success(),
    ) {
        (false, true, true) => stdout,
        (true, false, true) => stderr,
        (false, false, _) => format!("stdout:\n{stdout}\n\nstderr:\n{stderr}"),
        (false, true, false) => format!("stdout:\n{stdout}"),
        (true, false, false) => format!("stderr:\n{stderr}"),
        (true, true, true) => String::from("ok"),
        (true, true, false) => format!("command exited with {:?}", output.status.code()),
    }
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

fn error_response(id: Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
        }
    })
}

fn parse_tool_call(params: Option<Value>) -> Result<ToolCallRequest, SamsaraError> {
    let value =
        params.ok_or_else(|| SamsaraError::UpdateError("缺少 tools/call params".to_string()))?;
    serde_json::from_value(value)
        .map_err(|error| SamsaraError::UpdateError(format!("解析 tools/call 参数失败：{error}")))
}

fn required_string<'a>(arguments: &'a Value, key: &str) -> Result<&'a str, SamsaraError> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| SamsaraError::UpdateError(format!("缺少字符串参数：{key}")))
}

fn optional_u64(arguments: &Value, key: &str) -> Option<u64> {
    arguments.get(key).and_then(Value::as_u64)
}

fn optional_string_array(arguments: &Value, key: &str) -> Vec<String> {
    arguments
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default = "jsonrpc_version")]
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ToolCallRequest {
    name: String,
    #[serde(default = "empty_object")]
    arguments: Value,
}

fn jsonrpc_version() -> String {
    String::from("2.0")
}

fn empty_object() -> Value {
    json!({})
}
