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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-stone-900/40 dark:bg-black/70 backdrop-blur-md">
      <div className="w-[600px] rounded-2xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-2xl p-6 space-y-5 text-mono text-xs max-h-[85vh] flex flex-col transition-colors duration-200">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-stone-200 dark:border-stone-800">
          <div className="flex items-center space-x-2">
            <div className="p-1.5 rounded-lg bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-400">
              <FileJson className="w-4 h-4" />
            </div>
            <span className="font-bold font-display tracking-wider text-stone-900 dark:text-stone-100 uppercase">
              Session Snapshot
            </span>
          </div>
          <button
            onClick={toggleExport}
            className="p-1 rounded-lg hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-400 hover:text-stone-700 dark:hover:text-stone-200 transition-all"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Export Section */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-stone-600 dark:text-stone-400 font-bold">Export Active Session</span>
            <div className="flex items-center space-x-2">
              <button
                onClick={handleCopy}
                className="flex items-center space-x-1 px-2.5 py-1 rounded-lg bg-stone-100 dark:bg-stone-800 hover:bg-stone-200 dark:hover:bg-stone-700 text-stone-700 dark:text-stone-300 border border-stone-200 dark:border-stone-700 transition-all"
              >
                {copied ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copied ? "Copied" : "Copy JSON"}</span>
              </button>
              <button
                onClick={handleDownload}
                className="flex items-center space-x-1 px-2.5 py-1 rounded-lg bg-stone-900 hover:bg-stone-800 dark:bg-stone-100 dark:hover:bg-white text-white dark:text-stone-900 font-bold transition-all shadow-sm"
              >
                <Download className="w-3.5 h-3.5" />
                <span>Download .json</span>
              </button>
            </div>
          </div>

          <pre className="p-3.5 bg-stone-50 dark:bg-stone-950 rounded-xl border border-stone-200 dark:border-stone-800 text-[10px] text-stone-600 dark:text-stone-400 max-h-36 overflow-y-auto leading-relaxed">
            {jsonContent}
          </pre>
        </div>

        {/* Import Section */}
        <div className="space-y-2 pt-2 border-t border-stone-200 dark:border-stone-800">
          <span className="text-stone-600 dark:text-stone-400 font-bold">Import Session Snapshot</span>
          <textarea
            value={importText}
            onChange={(e) => setImportText(e.target.value)}
            placeholder="Paste exported session JSON here..."
            className="w-full h-24 p-3 bg-stone-50 dark:bg-stone-950 rounded-xl border border-stone-200 dark:border-stone-800 text-[11px] text-stone-900 dark:text-stone-100 placeholder:text-stone-400 outline-none resize-none font-mono focus:border-stone-500"
          />

          {errorMsg && (
            <div className="flex items-center space-x-1.5 text-rose-600 dark:text-rose-400 text-[11px]">
              <AlertCircle className="w-3.5 h-3.5" />
              <span>{errorMsg}</span>
            </div>
          )}

          {successMsg && (
            <div className="flex items-center space-x-1.5 text-emerald-600 dark:text-emerald-400 text-[11px]">
              <Check className="w-3.5 h-3.5" />
              <span>{successMsg}</span>
            </div>
          )}

          <div className="flex justify-end pt-2">
            <button
              onClick={handleImport}
              className="flex items-center space-x-1.5 px-4 py-1.5 rounded-xl bg-stone-900 hover:bg-stone-800 dark:bg-stone-100 dark:hover:bg-white text-white dark:text-stone-900 font-bold text-xs transition-all shadow-sm"
            >
              <Upload className="w-3.5 h-3.5" />
              <span>Restore Workspace</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
