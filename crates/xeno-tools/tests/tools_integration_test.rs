//! Comprehensive integration and unit tests for `crates/xeno-tools`.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use xeno_tools::prelude::*;

fn temp_workspace() -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("xeno_tools_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&p).unwrap();
    p
}

#[tokio::test]
async fn test_multi_replace_file_content_exact() {
    let ws = temp_workspace();
    let file = ws.join("sample.rs");

    let initial = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
    fs::write(&file, initial).unwrap();

    let tool = MultiReplaceTool::new();
    let ctx = ToolExecutionContext::default();

    let args = serde_json::json!({
        "TargetFile": file.to_string_lossy(),
        "TargetContent": "    a + b",
        "ReplacementContent": "    a + b + 10",
        "StartLine": 1,
        "EndLine": 3,
        "AllowMultiple": false
    });

    let res = tool.execute(args, &ctx).await.unwrap();
    assert!(res.success);
    assert!(res.diff_snippet.is_some());

    let content = fs::read_to_string(&file).unwrap();
    assert_eq!(content, "fn add(a: i32, b: i32) -> i32 {\n    a + b + 10\n}\n");

    let _ = fs::remove_dir_all(&ws);
}

#[tokio::test]
async fn test_multi_replace_ast_validation_rejects_invalid_syntax() {
    let ws = temp_workspace();
    let file = ws.join("sample_err.rs");

    let initial = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
    fs::write(&file, initial).unwrap();

    let tool = MultiReplaceTool::new();
    let ctx = ToolExecutionContext::default();

    // Invalid syntax: unbalanced paren
    let args = serde_json::json!({
        "TargetFile": file.to_string_lossy(),
        "TargetContent": "    a + b",
        "ReplacementContent": "    a + (b",
        "StartLine": 1,
        "EndLine": 3
    });

    let res = tool.execute(args, &ctx).await;
    assert!(res.is_err());

    // Ensure file was NOT modified
    let content = fs::read_to_string(&file).unwrap();
    assert_eq!(content, initial);

    let _ = fs::remove_dir_all(&ws);
}

#[tokio::test]
async fn test_atomic_write_and_slice_reader() {
    let ws = temp_workspace();
    let file = ws.join("data.txt");

    let write_tool = AtomicWriteTool::new();
    let read_tool = FileReadSliceTool::new();
    let ctx = ToolExecutionContext::default();

    let data = "Line 1: Alpha\nLine 2: Beta\nLine 3: Gamma\nLine 4: Delta\n";
    let write_args = serde_json::json!({
        "TargetFile": file.to_string_lossy(),
        "Content": data,
        "Overwrite": true
    });

    let w_res = write_tool.execute(write_args, &ctx).await.unwrap();
    assert!(w_res.success);

    // Read slice lines 2..3
    let read_args = serde_json::json!({
        "AbsolutePath": file.to_string_lossy(),
        "StartLine": 2,
        "EndLine": 3
    });

    let r_res = read_tool.execute(read_args, &ctx).await.unwrap();
    assert!(r_res.success);
    assert_eq!(r_res.stdout.trim(), "Line 2: Beta\nLine 3: Gamma");

    let _ = fs::remove_dir_all(&ws);
}

#[tokio::test]
async fn test_fuzzy_glob_ripgrep_tool() {
    let ws = temp_workspace();
    let file1 = ws.join("test_a.rs");
    let file2 = ws.join("test_b.py");

    fs::write(&file1, "pub fn search_target_func() {}\n").unwrap();
    fs::write(&file2, "def other_func(): pass\n").unwrap();

    let tool = FuzzyGlobRipgrepTool::new();
    let ctx = ToolExecutionContext::default();

    let args = serde_json::json!({
        "SearchPath": ws.to_string_lossy(),
        "Pattern": "*.rs",
        "Query": "search_target_func",
        "MatchPerLine": true
    });

    let res = tool.execute(args, &ctx).await.unwrap();
    assert!(res.success);
    assert!(res.stdout.contains("search_target_func"));
    assert!(!res.stdout.contains("other_func"));

    let _ = fs::remove_dir_all(&ws);
}

#[tokio::test]
async fn test_mcp_registry_registration_and_execution() {
    let mut registry = McpToolRegistry::new();
    registry.register_tool(Arc::new(MultiReplaceTool::new()));
    registry.register_tool(Arc::new(AtomicWriteTool::new()));
    registry.register_tool(Arc::new(FileReadSliceTool::new()));

    let tools = registry.list_mcp_tools();
    assert_eq!(tools.len(), 3);
    assert!(tools.iter().any(|t| t.name == "multi_replace_file_content"));
    assert!(tools.iter().any(|t| t.name == "atomic_write_file"));
    assert!(tools.iter().any(|t| t.name == "file_read_slice"));
}

#[tokio::test]
async fn test_security_classifier_tier_matrix() {
    let classifier = SecurityClassifier::new();

    assert_eq!(classifier.classify_command("git status"), SecurityTier::Tier1Safe);
    assert_eq!(classifier.classify_command("cargo test"), SecurityTier::Tier1Safe);
    assert_eq!(classifier.classify_command("git commit -m 'test'"), SecurityTier::Tier2Guarded);
    assert_eq!(classifier.classify_command("cargo add tokio"), SecurityTier::Tier2Guarded);
    assert_eq!(classifier.classify_command("rm -rf /"), SecurityTier::Tier3Destructive);
    assert_eq!(classifier.classify_command("del /s /q temp.txt"), SecurityTier::Tier3Destructive);
}
