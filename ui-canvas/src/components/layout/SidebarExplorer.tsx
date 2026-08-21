import React, { useState } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { 
  Folder, 
  FolderOpen, 
  FileCode, 
  Wrench, 
  FlaskConical, 
  ShieldCheck, 
  ChevronRight, 
  ChevronDown,
  CheckCircle2,
  Terminal as TerminalIcon,
  Bot
} from "lucide-react";

export const SidebarExplorer: React.FC = () => {
  const { isSidebarOpen, swarmAgents, activeView, setActiveView } = useWorkspaceStore();
  const [activeTab, setActiveTab] = useState<"files" | "tools" | "tests">("files");
  const [openFolders, setOpenFolders] = useState<Record<string, boolean>>({
    "crates": true,
    "crates/xeno-tools": true,
    "crates/xeno-router": true,
  });

  if (!isSidebarOpen) return null;

  const toggleFolder = (path: string) => {
    setOpenFolders((prev) => ({ ...prev, [path]: !prev[path] }));
  };

  const fileTree = [
    {
      name: "crates",
      isFolder: true,
      path: "crates",
      children: [
        {
          name: "xeno-core",
          isFolder: true,
          path: "crates/xeno-core",
          children: [
            { name: "contracts.rs", isFolder: false, path: "crates/xeno-core/src/contracts.rs" },
            { name: "events.rs", isFolder: false, path: "crates/xeno-core/src/events.rs" },
            { name: "errors.rs", isFolder: false, path: "crates/xeno-core/src/errors.rs" },
          ],
        },
        {
          name: "xeno-router",
          isFolder: true,
          path: "crates/xeno-router",
          children: [
            { name: "router.rs", isFolder: false, path: "crates/xeno-router/src/router.rs" },
            { name: "privacy.rs", isFolder: false, path: "crates/xeno-router/src/privacy.rs" },
            { name: "token_bus.rs", isFolder: false, path: "crates/xeno-router/src/token_bus.rs" },
          ],
        },
        {
          name: "xeno-tools",
          isFolder: true,
          path: "crates/xeno-tools",
          children: [
            { name: "ast_validator.rs", isFolder: false, path: "crates/xeno-tools/src/ast_validator.rs" },
            { name: "file_engine.rs", isFolder: false, path: "crates/xeno-tools/src/file_engine.rs" },
            { name: "pty.rs", isFolder: false, path: "crates/xeno-tools/src/pty.rs" },
            { name: "safety.rs", isFolder: false, path: "crates/xeno-tools/src/safety.rs" },
          ],
        },
        {
          name: "xeno-dag",
          isFolder: true,
          path: "crates/xeno-dag",
          children: [
            { name: "graph.rs", isFolder: false, path: "crates/xeno-dag/src/graph.rs" },
            { name: "node.rs", isFolder: false, path: "crates/xeno-dag/src/node.rs" },
          ],
        },
      ],
    },
  ];

  const mcpTools = [
    { name: "multi_replace_file_content", tier: "Tier 2", desc: "Line-bounded AST character exact replacement" },
    { name: "execute_sandboxed_command", tier: "Tier 1-3", desc: "Windows ConPTY execution with JobObject sandbox" },
    { name: "fuzzy_glob_search", tier: "Tier 1", desc: "Ripgrep accelerated token & symbol indexing" },
    { name: "ast_syntax_validate", tier: "Tier 1", desc: "Syn-driven syntax validator prior to commit" },
    { name: "tor_socks_fetch", tier: "Tier 1", desc: "Anonymized Tor onion socket resolver" },
  ];

  const testSuites = [
    { name: "xeno_core_tests", passed: 18, total: 18, status: "PASS" },
    { name: "xeno_router_tests", passed: 39, total: 39, status: "PASS" },
    { name: "xeno_tools_tests", passed: 15, total: 15, status: "PASS" },
    { name: "xeno_dag_tests", passed: 14, total: 14, status: "PASS" },
    { name: "e2e_vertical_slice", passed: 18, total: 18, status: "PASS" },
  ];

  return (
    <aside className="w-64 border-r border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 flex flex-col justify-between text-xs font-mono select-none z-30 transition-colors duration-200">
      {/* Top Tab Bar */}
      <div>
        <div className="h-10 border-b border-stone-200 dark:border-stone-800 flex items-center bg-stone-50 dark:bg-stone-950">
          <button
            onClick={() => setActiveTab("files")}
            className={`flex-1 h-full flex items-center justify-center space-x-1 border-r border-stone-200 dark:border-stone-800 transition-all ${
              activeTab === "files" ? "bg-white dark:bg-stone-900 text-stone-900 dark:text-stone-100 font-bold border-b-2 border-b-stone-900 dark:border-b-stone-100" : "text-stone-500 hover:text-stone-800 dark:hover:text-stone-200"
            }`}
          >
            <FileCode className="w-3.5 h-3.5" />
            <span>Files</span>
          </button>
          <button
            onClick={() => setActiveTab("tools")}
            className={`flex-1 h-full flex items-center justify-center space-x-1 border-r border-stone-200 dark:border-stone-800 transition-all ${
              activeTab === "tools" ? "bg-white dark:bg-stone-900 text-stone-900 dark:text-stone-100 font-bold border-b-2 border-b-stone-900 dark:border-b-stone-100" : "text-stone-500 hover:text-stone-800 dark:hover:text-stone-200"
            }`}
          >
            <Wrench className="w-3.5 h-3.5" />
            <span>MCP Tools</span>
          </button>
          <button
            onClick={() => setActiveTab("tests")}
            className={`flex-1 h-full flex items-center justify-center space-x-1 transition-all ${
              activeTab === "tests" ? "bg-white dark:bg-stone-900 text-stone-900 dark:text-stone-100 font-bold border-b-2 border-b-stone-900 dark:border-b-stone-100" : "text-stone-500 hover:text-stone-800 dark:hover:text-stone-200"
            }`}
          >
            <FlaskConical className="w-3.5 h-3.5" />
            <span>Tests</span>
          </button>
        </div>

        {/* Tab Content */}
        <div className="p-3 overflow-y-auto max-h-[calc(100vh-14rem)] space-y-1">
          {activeTab === "files" && (
            <div className="space-y-1">
              {fileTree[0].children.map((crate) => {
                const isOpen = openFolders[crate.path];
                return (
                  <div key={crate.path} className="space-y-0.5">
                    <div
                      onClick={() => toggleFolder(crate.path)}
                      className="flex items-center space-x-1.5 py-1 px-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 cursor-pointer text-stone-700 dark:text-stone-300 transition-all"
                    >
                      {isOpen ? <ChevronDown className="w-3.5 h-3.5 text-stone-400" /> : <ChevronRight className="w-3.5 h-3.5 text-stone-400" />}
                      <Folder className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
                      <span className="font-semibold">{crate.name}</span>
                    </div>

                    {isOpen && (
                      <div className="pl-4 space-y-0.5">
                        {crate.children.map((file) => (
                          <div
                            key={file.path}
                            onClick={() => setActiveView("diff")}
                            className="flex items-center space-x-1.5 py-1 px-1.5 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 cursor-pointer text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-100 transition-all"
                          >
                            <FileCode className="w-3.5 h-3.5 text-stone-400" />
                            <span>{file.name}</span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}

          {activeTab === "tools" && (
            <div className="space-y-2">
              {mcpTools.map((tool) => (
                <div
                  key={tool.name}
                  className="p-2.5 rounded-xl border border-stone-200 dark:border-stone-800 bg-stone-50/70 dark:bg-stone-950 space-y-1"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-stone-800 dark:text-stone-200 truncate max-w-[140px]">
                      {tool.name}
                    </span>
                    <span className="text-[9px] px-1.5 py-0.2 rounded bg-stone-200 dark:bg-stone-800 text-stone-600 dark:text-stone-400 font-bold">
                      {tool.tier}
                    </span>
                  </div>
                  <p className="text-[10px] text-stone-500 dark:text-stone-400 font-sans leading-tight">
                    {tool.desc}
                  </p>
                </div>
              ))}
            </div>
          )}

          {activeTab === "tests" && (
            <div className="space-y-2">
              <div className="p-2 rounded-xl bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/60 flex items-center justify-between">
                <div className="flex items-center space-x-1.5 text-emerald-700 dark:text-emerald-300">
                  <CheckCircle2 className="w-4 h-4" />
                  <span className="font-bold">120/120 Passed</span>
                </div>
                <span className="text-[10px] text-emerald-600 dark:text-emerald-400 font-bold">100%</span>
              </div>

              {testSuites.map((ts) => (
                <div
                  key={ts.name}
                  className="p-2 rounded-xl border border-stone-200 dark:border-stone-800 bg-stone-50 dark:bg-stone-950 flex items-center justify-between text-[11px]"
                >
                  <span className="text-stone-700 dark:text-stone-300 font-mono truncate">{ts.name}</span>
                  <span className="text-emerald-600 dark:text-emerald-400 font-bold">{ts.passed}/{ts.total}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Active Agents Snapshot */}
      <div className="p-3 border-t border-stone-200 dark:border-stone-800 bg-stone-50 dark:bg-stone-950 space-y-2">
        <div className="flex items-center justify-between text-[10px] text-stone-500 uppercase tracking-wider font-bold">
          <span>Active Council</span>
          <span className="text-emerald-600 dark:text-emerald-400">5 Ready</span>
        </div>

        <div className="flex -space-x-1 overflow-hidden">
          {swarmAgents.map((ag) => (
            <div
              key={ag.role}
              onClick={() => setActiveView("swarm")}
              className="w-7 h-7 rounded-full border-2 border-white dark:border-stone-900 bg-stone-200 dark:bg-stone-800 flex items-center justify-center text-[10px] font-bold text-stone-700 dark:text-stone-300 cursor-pointer hover:scale-110 transition-transform shadow-sm"
              title={`${ag.title} (${ag.model})`}
            >
              {ag.role[0].toUpperCase()}
            </div>
          ))}
        </div>
      </div>
    </aside>
  );
};
