import React, { useState } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Download, Upload, Copy, Check, X, FileJson, AlertCircle } from "lucide-react";

export const SessionExportModal: React.FC = () => {
  const { isExportOpen, toggleExport, exportSessionJson, importSessionJson } = useWorkspaceStore();
  const [copied, setCopied] = useState(false);
  const [importText, setImportText] = useState("");
  const [errorMsg, setErrorMsg] = useState("");
  const [successMsg, setSuccessMsg] = useState("");

  if (!isExportOpen) return null;

  const jsonContent = exportSessionJson();

  const handleCopy = () => {
    navigator.clipboard.writeText(jsonContent);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const handleDownload = () => {
    const blob = new Blob([jsonContent], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `xeno-session-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = () => {
    setErrorMsg("");
    setSuccessMsg("");
    if (!importText.trim()) {
      setErrorMsg("Please paste valid JSON snapshot.");
      return;
    }
    const success = importSessionJson(importText);
    if (success) {
      setSuccessMsg("Session imported successfully!");
      setTimeout(() => {
        toggleExport();
      }, 1000);
    } else {
      setErrorMsg("Invalid session JSON format. Expected canvasNodes and dagNodes.");
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md">
      <div className="w-[600px] rounded-2xl border border-border-700 bg-surface-900 shadow-2xl p-6 space-y-5 text-mono text-xs max-h-[85vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-border-700">
          <div className="flex items-center space-x-2">
            <div className="p-1.5 rounded-lg bg-emerald-500/10 text-emerald-400">
              <FileJson className="w-4 h-4" />
            </div>
            <span className="font-bold text-neutral-100 uppercase tracking-wider">Session State Snapshot</span>
          </div>
          <button
            onClick={toggleExport}
            className="p-1 rounded-lg hover:bg-surface-800 text-neutral-400 hover:text-neutral-200 transition-all"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Export Section */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-neutral-400 font-bold">Export Active Session</span>
            <div className="flex items-center space-x-2">
              <button
                onClick={handleCopy}
                className="flex items-center space-x-1 px-2.5 py-1 rounded bg-surface-800 hover:bg-surface-700 text-neutral-200 border border-border-700 transition-all"
              >
                {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copied ? "Copied" : "Copy JSON"}</span>
              </button>
              <button
                onClick={handleDownload}
                className="flex items-center space-x-1 px-2.5 py-1 rounded bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 transition-all"
              >
                <Download className="w-3.5 h-3.5" />
                <span>Download .json</span>
              </button>
            </div>
          </div>

          <pre className="p-3 bg-surface-950 rounded-xl border border-border-700 text-[10px] text-neutral-400 max-h-36 overflow-y-auto leading-relaxed">
            {jsonContent}
          </pre>
        </div>

        {/* Import Section */}
        <div className="space-y-2 pt-2 border-t border-border-700">
          <span className="text-neutral-400 font-bold">Import Session Snapshot</span>
          <textarea
            value={importText}
            onChange={(e) => setImportText(e.target.value)}
            placeholder="Paste exported session JSON here..."
            className="w-full h-24 p-3 bg-surface-950 rounded-xl border border-border-700 text-[11px] text-neutral-200 placeholder:text-neutral-600 outline-none resize-none font-mono focus:border-cyan-500/50"
          />

          {errorMsg && (
            <div className="flex items-center space-x-1.5 text-rose-400 text-[11px]">
              <AlertCircle className="w-3.5 h-3.5" />
              <span>{errorMsg}</span>
            </div>
          )}

          {successMsg && (
            <div className="flex items-center space-x-1.5 text-emerald-400 text-[11px]">
              <Check className="w-3.5 h-3.5" />
              <span>{successMsg}</span>
            </div>
          )}

          <div className="flex justify-end pt-2">
            <button
              onClick={handleImport}
              className="flex items-center space-x-1.5 px-4 py-1.5 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-surface-950 font-bold text-xs transition-all shadow-lg glow-cyan"
            >
              <Upload className="w-3.5 h-3.5" />
              <span>Restore Workspace State</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
