import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { Keyboard, X } from "lucide-react";

export const ShortcutsModal: React.FC = () => {
  const { isShortcutsOpen, toggleShortcuts } = useWorkspaceStore();

  if (!isShortcutsOpen) return null;

  const shortcuts = [
    { key: "1 - 6", desc: "Switch view mode (Canvas, DAG, Timeline, Terminal, Diff, Swarm)" },
    { key: "Ctrl + K", desc: "Focus Command OmniBar" },
    { key: "Space + Drag", desc: "Pan Spatial Canvas infinite viewport" },
    { key: "Ctrl + Enter", desc: "Execute current prompt or shell command" },
    { key: "Shift + ?", desc: "Toggle this Keyboard Shortcuts guide" },
    { key: "Ctrl + E", desc: "Open Session Snapshot Export / Import modal" },
    { key: "Ctrl + B", desc: "Toggle Sidebar Explorer visibility" },
    { key: "Esc", desc: "Close active modal / deselect node" },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md">
      <div className="w-[500px] rounded-2xl border border-border-700 bg-surface-900 shadow-2xl p-6 space-y-5 text-mono text-xs">
        <div className="flex items-center justify-between pb-3 border-b border-border-700">
          <div className="flex items-center space-x-2">
            <div className="p-1.5 rounded-lg bg-cyan-500/10 text-cyan-400">
              <Keyboard className="w-4 h-4" />
            </div>
            <span className="font-bold text-neutral-100 uppercase tracking-wider">Keyboard Shortcuts</span>
          </div>
          <button
            onClick={toggleShortcuts}
            className="p-1 rounded-lg hover:bg-surface-800 text-neutral-400 hover:text-neutral-200 transition-all"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="space-y-2.5">
          {shortcuts.map((sc, i) => (
            <div key={i} className="flex items-center justify-between py-1.5 px-2.5 rounded-lg bg-surface-950/60 border border-border-700/60">
              <span className="text-neutral-300">{sc.desc}</span>
              <kbd className="px-2 py-0.5 rounded bg-surface-800 border border-border-700 text-cyan-400 font-bold font-mono text-[11px]">
                {sc.key}
              </kbd>
            </div>
          ))}
        </div>

        <div className="pt-2 text-[10px] text-neutral-500 text-center font-mono">
          Press <kbd className="text-neutral-400">Esc</kbd> anytime to dismiss.
        </div>
      </div>
    </div>
  );
};
