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
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex flex-col bg-stone-50 dark:bg-stone-950 text-xs font-mono select-text transition-colors duration-200">
      {/* Terminal Top Bar */}
      <div className="h-12 border-b border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 px-4 flex items-center justify-between z-10 shadow-sm">
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-2">
            <div className="w-2.5 h-2.5 rounded-full bg-emerald-500" />
            <span className="font-bold text-stone-900 dark:text-stone-100 font-display">PTY SESSION #pty-8492</span>
          </div>
          <span className="text-[10px] px-2 py-0.5 rounded-md bg-stone-100 dark:bg-stone-800 border border-stone-200 dark:border-stone-700 text-stone-600 dark:text-stone-400">
            Windows ConPTY • JobObject Isolated
          </span>
        </div>

        <div className="flex items-center space-x-3">
          <span className="flex items-center gap-1.5 text-[10px] px-2.5 py-0.5 rounded-md bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300 border border-stone-200 dark:border-stone-700 font-bold">
            <Shield className="w-3 h-3 text-emerald-600 dark:text-emerald-400" />
            {securityTier}
          </span>
        </div>
      </div>

      {/* Terminal Output Area */}
      <div className="flex-1 p-6 overflow-y-auto space-y-2 bg-stone-100/50 dark:bg-stone-950">
        {terminalLogs.map((log) => {
          let textStyle = "text-stone-800 dark:text-stone-200";
          let badge = null;

          if (log.type === "command") {
            textStyle = "text-stone-950 dark:text-stone-50 font-bold";
          } else if (log.type === "stderr") {
            textStyle = "text-rose-600 dark:text-rose-400";
          } else if (log.type === "intervention") {
            textStyle = "text-emerald-800 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-950/40 p-3 rounded-xl border border-emerald-200 dark:border-emerald-800 my-2";
            badge = <CheckCircle2 className="w-4 h-4 text-emerald-600 dark:text-emerald-400 inline mr-1.5" />;
          } else if (log.type === "system") {
            textStyle = "text-stone-500 dark:text-stone-400";
          }

          return (
            <div key={log.id} className="flex items-start space-x-3 leading-relaxed">
              <span className="text-stone-400 select-none text-[10px] min-w-[50px]">{log.timestamp}</span>
              <div className={`flex-1 break-all ${textStyle}`}>
                {badge}
                {log.content}
              </div>
            </div>
          );
        })}
        <div ref={endRef} />
      </div>

      {/* Terminal Input Line */}
      <form
        onSubmit={handleSubmit}
        className="h-12 border-t border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 px-4 flex items-center space-x-2"
      >
        <span className="text-emerald-600 dark:text-emerald-400 font-bold text-sm">$</span>
        <input
          type="text"
          value={cmdInput}
          onChange={(e) => setCmdInput(e.target.value)}
          placeholder="Execute shell command in Windows ConPTY sandbox..."
          className="flex-1 bg-transparent text-xs text-stone-900 dark:text-stone-100 placeholder:text-stone-400 outline-none font-mono"
        />
        <button
          type="submit"
          className="px-3 py-1 bg-stone-900 hover:bg-stone-800 dark:bg-stone-100 dark:hover:bg-white text-white dark:text-stone-900 font-bold rounded-lg text-xs transition-all shadow-sm"
        >
          Execute
        </button>
      </form>
    </div>
  );
};
