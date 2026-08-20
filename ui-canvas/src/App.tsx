import React, { useState } from "react";
import { PromptNode } from "./components/PromptNode";
import { SubagentNode } from "./components/SubagentNode";
import { DiffNode } from "./components/DiffNode";
import { TelemetryHUD } from "./components/TelemetryHUD";

export const App: React.FC = () => {
  const [prompt, setPrompt] = useState("");

  return (
    <div className="w-screen h-screen bg-neutral-950 text-neutral-100 flex flex-col justify-between p-6 relative overflow-hidden font-sans">
      <TelemetryHUD
        velocity={138.4}
        costUsd={0.0182}
        vramUsedGb={14.8}
        vramTotalGb={24.0}
        ttftMs={34}
      />

      {/* Top Banner */}
      <header className="flex items-center justify-between border-b border-neutral-900 pb-4">
        <div className="flex items-center space-x-3">
          <div className="w-3 h-3 rounded-full bg-cyan-400 animate-pulse" />
          <h1 className="text-sm font-bold tracking-widest uppercase font-mono text-cyan-400">
            XENO INFERENCE // SPATIAL CANVAS
          </h1>
        </div>
        <div className="text-xs font-mono text-neutral-500">TAURI V2 + REACT 19</div>
      </header>

      {/* Central Canvas Viewport */}
      <main className="flex-1 relative flex items-center justify-center space-x-8">
        <PromptNode
          id="node-1"
          label="User Instruction"
          content="Implement AST validation engine with multi-replace file editing"
          status="completed"
        />
        <div className="w-8 h-[2px] bg-gradient-to-r from-cyan-500 to-emerald-500" />
        <SubagentNode
          id="node-2"
          role="coder"
          model="claude-3-7-sonnet"
          phase="Act: AST Patching"
          progress={100}
        />
        <div className="w-8 h-[2px] bg-gradient-to-r from-emerald-500 to-amber-500" />
        <DiffNode
          id="node-3"
          filePath="crates/xeno-tools/src/ast_validator.rs"
          diffContent="--- original\n+++ replacement\n-pub fn validate() {}\n+pub fn validate_syntax(&self, path: &Path, code: &str) -> Result<(), ToolError> {}"
        />
      </main>

      {/* Bottom Omni-Bar */}
      <footer className="w-full max-w-3xl mx-auto">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setPrompt("");
          }}
          className="relative flex items-center"
        >
          <input
            type="text"
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Type prompt or command (/swarm, /dag, /diff)..."
            className="w-full bg-neutral-900/90 border border-neutral-800 focus:border-cyan-500 rounded-xl px-5 py-3.5 text-sm text-neutral-200 outline-none backdrop-blur-xl transition-all shadow-2xl font-mono"
          />
          <button
            type="submit"
            className="absolute right-2 px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-xs font-mono font-semibold transition-all"
          >
            RUN
          </button>
        </form>
      </footer>
    </div>
  );
};

export default App;
