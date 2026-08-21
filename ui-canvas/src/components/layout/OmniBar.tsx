import React, { useState } from "react";
import { useWorkspaceStore, RoutingPolicy } from "../../stores/workspaceStore";
import { Zap, Brain, Shield, Coins, Send, Terminal, Users, Sparkles, Globe } from "lucide-react";

export const OmniBar: React.FC = () => {
  const [input, setInput] = useState("");
  const { 
    routingPolicy, 
    setRoutingPolicy, 
    handleSmartPrompt
  } = useWorkspaceStore();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;
    handleSmartPrompt(input);
    setInput("");
  };

  const policies: { id: RoutingPolicy; label: string; icon: React.ReactNode }[] = [
    { id: "speed", label: "Speed (<50ms)", icon: <Zap className="w-3 h-3 text-amber-600 dark:text-amber-400" /> },
    { id: "reasoning", label: "Deep Reasoning", icon: <Brain className="w-3 h-3 text-purple-600 dark:text-purple-400" /> },
    { id: "privacy", label: "Air-Gap Guard", icon: <Shield className="w-3 h-3 text-emerald-600 dark:text-emerald-400" /> },
    { id: "cost", label: "Cost-Optimized", icon: <Coins className="w-3 h-3 text-stone-600 dark:text-stone-400" /> },
  ];

  return (
    <footer className="fixed bottom-6 left-1/2 -translate-x-1/2 w-full max-w-4xl z-40 px-4">
      <div className="p-2 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 backdrop-blur-2xl shadow-xl space-y-2 transition-colors duration-200">
        {/* Top Control Bar: Routing Policies & Slash Shortcut Pills */}
        <div className="flex items-center justify-between px-2 text-[11px] font-mono">
          {/* Routing Policy Selector */}
          <div className="flex items-center space-x-1.5">
            <span className="text-stone-400 dark:text-stone-500 uppercase text-[9px] font-bold">Policy:</span>
            {policies.map((p) => {
              const isSelected = routingPolicy === p.id;
              return (
                <button
                  key={p.id}
                  onClick={() => setRoutingPolicy(p.id)}
                  className={`flex items-center space-x-1 px-2 py-0.5 rounded-lg border transition-all ${
                    isSelected 
                      ? "bg-stone-100 dark:bg-stone-800 border-stone-300 dark:border-stone-700 text-stone-900 dark:text-stone-100 font-bold shadow-sm" 
                      : "border-transparent text-stone-500 hover:text-stone-900 dark:hover:text-stone-200 hover:bg-stone-50 dark:hover:bg-stone-850"
                  }`}
                >
                  {p.icon}
                  <span>{p.label}</span>
                </button>
              );
            })}
          </div>

          {/* Quick Shortcuts */}
          <div className="hidden sm:flex items-center space-x-2 text-stone-500">
            <button
              onClick={() => handleSmartPrompt("browse onion site")}
              className="flex items-center space-x-1 hover:text-stone-900 dark:hover:text-stone-200"
            >
              <Globe className="w-3 h-3" />
              <span>/tor</span>
            </button>
            <span>•</span>
            <button
              onClick={() => handleSmartPrompt("launch swarm council")}
              className="flex items-center space-x-1 hover:text-stone-900 dark:hover:text-stone-200"
            >
              <Users className="w-3 h-3" />
              <span>/swarm</span>
            </button>
            <span>•</span>
            <button
              onClick={() => handleSmartPrompt("execute cargo test in terminal")}
              className="flex items-center space-x-1 hover:text-stone-900 dark:hover:text-stone-200"
            >
              <Terminal className="w-3 h-3" />
              <span>/term</span>
            </button>
          </div>
        </div>

        {/* Input Bar */}
        <form onSubmit={handleSubmit} className="flex items-center space-x-2 bg-stone-50 dark:bg-stone-950 p-1.5 rounded-xl border border-stone-200 dark:border-stone-800">
          <div className="pl-2.5 text-stone-400">
            <Sparkles className="w-4 h-4 text-amber-600 dark:text-amber-400" />
          </div>

          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Type directive, shell command, or natural language query (Ctrl + K to focus)..."
            className="flex-1 bg-transparent px-2 py-1.5 text-xs text-stone-900 dark:text-stone-100 placeholder:text-stone-400 font-sans outline-none"
          />

          <button
            type="submit"
            className="px-3 py-1.5 rounded-lg bg-stone-900 hover:bg-stone-800 dark:bg-stone-100 dark:hover:bg-white text-white dark:text-stone-900 text-xs font-mono font-bold flex items-center space-x-1.5 transition-all shadow-sm"
          >
            <span>Run</span>
            <Send className="w-3 h-3" />
          </button>
        </form>
      </div>
    </footer>
  );
};
