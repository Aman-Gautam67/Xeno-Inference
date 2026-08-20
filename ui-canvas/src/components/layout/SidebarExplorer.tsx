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
            { name: "pty.rs", isFolder: false, path: "crates/xeno-tools/src/pty.rs" },
            { name: "file_engine.rs", isFolder: false, path: "crates/xeno-tools/src/file_engine.rs" },
          ],
        },
        {
          name: "xeno-agent",
          isFolder: true,
          path: "crates/xeno-agent",
          children: [
            { name: "harness.rs", isFolder: false, path: "crates/xeno-agent/src/harness.rs" },
            { name: "swarm.rs", isFolder: false, path: "crates/xeno-agent/src/swarm.rs" },
            { name: "paorv.rs", isFolder: false, path: "crates/xeno-agent/src/paorv.rs" },
          ],
        },
      ],
    },
  ];

  const mcpTools = [
    { name: "terminal_exec", desc: "Sandboxed ConPTY execution", tier: "Tier 1/2", active: true },
    { name: "multi_replace_file", desc: "Atomic AST multi-replace", tier: "Tier 2", active: true },
    { name: "lsp_diagnostics", desc: "rust-analyzer type checker", tier: "Tier 1", active: true },
    { name: "fuzzy_glob_ripgrep", desc: "High-speed codebase search", tier: "Tier 1", active: true },
    { name: "git_autopilot", desc: "Worktree branch manager", tier: "Tier 2", active: true },
  ];

  return (
    <aside className="w-64 border-r border-border-700 bg-surface-900/95 flex flex-col h-[calc(100vh-3.5rem)] text-xs font-mono select-none z-30">
      {/* Sidebar Tabs */}
      <div className="flex border-b border-border-700 bg-surface-950/60 p-1">
        <button
          onClick={() => setActiveTab("files")}
          className={`flex-1 py-1.5 flex items-center justify-center space-x-1.5 rounded-md transition-all ${
            activeTab === "files" ? "bg-surface-800 text-cyan-400 font-semibold" : "text-neutral-500 hover:text-neutral-300"
          }`}
        >
          <Folder className="w-3.5 h-3.5" />
          <span>Files</span>
        </button>
        <button
          onClick={() => setActiveTab("tools")}
          className={`flex-1 py-1.5 flex items-center justify-center space-x-1.5 rounded-md transition-all ${
            activeTab === "tools" ? "bg-surface-800 text-cyan-400 font-semibold" : "text-neutral-500 hover:text-neutral-300"
          }`}
        >
          <Wrench className="w-3.5 h-3.5" />
          <span>MCP Tools</span>
        </button>
        <button
          onClick={() => setActiveTab("tests")}
          className={`flex-1 py-1.5 flex items-center justify-center space-x-1.5 rounded-md transition-all ${
            activeTab === "tests" ? "bg-surface-800 text-emerald-400 font-semibold" : "text-neutral-500 hover:text-neutral-300"
          }`}
        >
          <FlaskConical className="w-3.5 h-3.5" />
          <span>Tests</span>
        </button>
      </div>

      {/* Explorer Content */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {activeTab === "files" && (
          <div className="space-y-1">
            <div className="text-[10px] uppercase tracking-wider text-neutral-500 mb-2 font-semibold">Workspace Crates</div>
            {fileTree.map((root) => (
              <div key={root.path} className="space-y-1">
                <div 
                  onClick={() => toggleFolder(root.path)}
                  className="flex items-center space-x-1.5 p-1 rounded hover:bg-surface-800 text-neutral-300 cursor-pointer"
                >
                  {openFolders[root.path] ? <ChevronDown className="w-3 h-3 text-neutral-500" /> : <ChevronRight className="w-3 h-3 text-neutral-500" />}
                  {openFolders[root.path] ? <FolderOpen className="w-3.5 h-3.5 text-cyan-400" /> : <Folder className="w-3.5 h-3.5 text-neutral-400" />}
                  <span>{root.name}</span>
                </div>

                {openFolders[root.path] && (
                  <div className="pl-4 space-y-1">
                    {root.children?.map((crate) => (
                      <div key={crate.path} className="space-y-0.5">
                        <div 
                          onClick={() => toggleFolder(crate.path)}
                          className="flex items-center space-x-1.5 p-1 rounded hover:bg-surface-800 text-neutral-300 cursor-pointer"
                        >
                          {openFolders[crate.path] ? <ChevronDown className="w-3 h-3 text-neutral-500" /> : <ChevronRight className="w-3 h-3 text-neutral-500" />}
                          <Folder className="w-3.5 h-3.5 text-purple-400" />
                          <span>{crate.name}</span>
                        </div>

                        {openFolders[crate.path] && (
                          <div className="pl-4 space-y-0.5">
                            {crate.children?.map((file) => (
                              <div
                                key={file.path}
                                onClick={() => setActiveView("diff")}
                                className="flex items-center space-x-1.5 p-1 rounded hover:bg-surface-800 text-neutral-400 hover:text-cyan-300 cursor-pointer"
                              >
                                <FileCode className="w-3 h-3 text-cyan-400/70" />
                                <span className="truncate">{file.name}</span>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        {activeTab === "tools" && (
          <div className="space-y-2.5">
            <div className="text-[10px] uppercase tracking-wider text-neutral-500 font-semibold">Registered MCP Tools</div>
            {mcpTools.map((t) => (
              <div key={t.name} className="p-2 rounded-lg border border-border-700 bg-surface-800/60 space-y-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-cyan-300">{t.name}</span>
                  <span className="text-[9px] px-1.5 py-0.2 rounded bg-surface-950 text-amber-400 border border-border-700">
                    {t.tier}
                  </span>
                </div>
                <div className="text-[10px] text-neutral-400">{t.desc}</div>
              </div>
            ))}
          </div>
        )}

        {activeTab === "tests" && (
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-[10px] uppercase tracking-wider text-neutral-500 font-semibold">Test Matrix</span>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
                120 / 120 PASS
              </span>
            </div>

            <div className="space-y-1.5">
              {[
                { name: "e2e_vertical_slice", passed: true, time: "0.01s" },
                { name: "e2e_workloads", passed: true, time: "0.02s" },
                { name: "e2e_boundary_tests", passed: true, time: "0.01s" },
                { name: "e2e_opaque_box", passed: true, time: "0.01s" },
                { name: "empirical_challenge_m2", passed: true, time: "0.48s" },
              ].map((test) => (
                <div key={test.name} className="p-2 rounded border border-border-700 bg-surface-850 flex items-center justify-between text-[11px]">
                  <div className="flex items-center space-x-1.5">
                    <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                    <span className="text-neutral-300 truncate max-w-[130px]">{test.name}</span>
                  </div>
                  <span className="text-neutral-500 text-[10px]">{test.time}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Swarm Agents Mini Footer */}
      <div className="p-3 border-t border-border-700 bg-surface-950/80 space-y-2">
        <div className="flex items-center justify-between text-[10px] text-neutral-500 font-semibold uppercase">
          <span className="flex items-center gap-1">
            <Bot className="w-3.5 h-3.5 text-cyan-400" /> Active Swarm
          </span>
          <span className="text-cyan-400">5 Roles</span>
        </div>
        <div className="grid grid-cols-5 gap-1">
          {swarmAgents.map((a) => (
            <div
              key={a.role}
              onClick={() => setActiveView("swarm")}
              className="p-1 rounded bg-surface-800 border border-border-700 text-center cursor-pointer hover:border-cyan-400 transition-all"
              title={`${a.title} (${a.status})`}
            >
              <div className={`w-2 h-2 mx-auto rounded-full ${
                a.status === "coding" ? "bg-cyan-400 animate-ping" : a.status === "planning" ? "bg-purple-400" : "bg-emerald-400"
              }`} />
              <span className="text-[8px] uppercase text-neutral-400">{a.role.slice(0, 3)}</span>
            </div>
          ))}
        </div>
      </div>
    </aside>
  );
};
