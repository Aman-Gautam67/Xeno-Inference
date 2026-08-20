import React, { useState, useRef, useEffect } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Terminal, Shield, Play, Trash2, CheckCircle2, AlertTriangle, ShieldCheck } from "lucide-react";

export const SandboxedTerminalView: React.FC = () => {
  const { terminalLogs, executeCommand, securityTier } = useWorkspaceStore();
  const [cmdInput, setCmdInput] = useState("");
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [terminalLogs]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!cmdInput.trim()) return;
    executeCommand(cmdInput);
    setCmdInput("");
  };

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex flex-col bg-surface-950 text-xs font-mono select-text">
      {/* Terminal Top Bar */}
      <div className="h-10 border-b border-border-700 bg-surface-900 px-4 flex items-center justify-between z-10">
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-emerald-400" />
            <span className="font-bold text-neutral-200">PTY SESSION #pty-8492</span>
          </div>
          <span className="text-[10px] px-2 py-0.5 rounded bg-surface-950 border border-border-700 text-neutral-400">
            Windows ConPTY • JobObject Isolation
          </span>
        </div>

        <div className="flex items-center space-x-3">
          <span className="flex items-center gap-1.5 text-[10px] px-2.5 py-0.5 rounded bg-amber-500/10 text-amber-300 border border-amber-500/30 font-semibold">
            <Shield className="w-3 h-3" />
            {securityTier}
          </span>
        </div>
      </div>

      {/* Terminal Output Area */}
      <div className="flex-1 p-4 overflow-y-auto space-y-2 bg-void scanlines">
        {terminalLogs.map((log) => {
          let textStyle = "text-neutral-300";
          let badge = null;

          if (log.type === "command") {
            textStyle = "text-cyan-400 font-bold";
          } else if (log.type === "stderr") {
            textStyle = "text-rose-400";
          } else if (log.type === "intervention") {
            textStyle = "text-emerald-300 bg-emerald-950/30 p-2.5 rounded-xl border border-emerald-500/30 my-2";
            badge = <CheckCircle2 className="w-4 h-4 text-emerald-400 inline mr-1.5" />;
          } else if (log.type === "system") {
            textStyle = "text-neutral-500";
          }

          return (
            <div key={log.id} className="leading-relaxed">
              <span className="text-neutral-600 mr-2">[{log.timestamp}]</span>
              {badge}
              <span className={textStyle}>
                {log.content}
              </span>
            </div>
          );
        })}
        <div ref={endRef} />
      </div>

      {/* Terminal Command Input */}
      <form
        onSubmit={handleSubmit}
        className="h-12 border-t border-border-700 bg-surface-900 px-4 flex items-center space-x-2"
      >
        <span className="text-cyan-400 font-bold">xeno-pty $</span>
        <input
          type="text"
          value={cmdInput}
          onChange={(e) => setCmdInput(e.target.value)}
          placeholder="Run shell command inside virtual ConPTY sandbox..."
          className="flex-1 bg-transparent text-neutral-100 placeholder:text-neutral-600 outline-none font-mono text-xs"
        />
        <button
          type="submit"
          className="px-3 py-1 bg-surface-800 hover:bg-cyan-500 hover:text-neutral-950 text-neutral-300 rounded border border-border-700 text-[11px] transition-all"
        >
          Execute
        </button>
      </form>
    </div>
  );
};
