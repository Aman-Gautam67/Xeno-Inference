import React, { useState } from "react";
import { useWorkspaceStore, RoutingPolicy } from "../../stores/workspaceStore";
import { Zap, Brain, Shield, Coins, Send, Terminal, Users, Sparkles } from "lucide-react";

export const OmniBar: React.FC = () => {
  const [input, setInput] = useState("");
  const { 
    routingPolicy, 
    setRoutingPolicy, 
    executeCommand, 
    dispatchSwarmTask,
    setActiveView 
  } = useWorkspaceStore();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;

    if (input.startsWith("/swarm")) {
      const task = input.replace("/swarm", "").trim() || "Analyze and refactor codebase components";
      dispatchSwarmTask(task);
    } else if (input === "/dag") {
      setActiveView("dag");
    } else if (input === "/diff") {
      setActiveView("diff");
    } else if (input === "/terminal") {
      setActiveView("terminal");
    } else if (input === "/timeline") {
      setActiveView("timeline");
    } else {
      executeCommand(input);
    }
    setInput("");
  };

  const policies: { id: RoutingPolicy; label: string; icon: React.ReactNode }[] = [
    { id: "speed", label: "Speed (<50ms)", icon: <Zap className="w-3 h-3 text-cyan-400" /> },
    { id: "reasoning", label: "Deep Reasoning", icon: <Brain className="w-3 h-3 text-purple-400" /> },
    { id: "privacy", label: "Air-Gap Guard", icon: <Shield className="w-3 h-3 text-emerald-400" /> },
    { id: "cost", label: "Cost-Optimized", icon: <Coins className="w-3 h-3 text-amber-400" /> },
  ];

  return (
    <footer className="fixed bottom-6 left-1/2 -translate-x-1/2 w-full max-w-4xl z-40 px-4">
      <div className="p-2 rounded-2xl border border-border-700 bg-surface-900/90 backdrop-blur-2xl shadow-2xl space-y-2">
        {/* Top Control Bar: Routing Policies & Slash Shortcut Pills */}
        <div className="flex items-center justify-between px-2 text-[11px] font-mono">
          {/* Routing Policy Selector */}
          <div className="flex items-center space-x-1.5">
            <span className="text-neutral-500 uppercase text-[9px]">Policy:</span>
            {policies.map((p) => {
              const isSelected = routingPolicy === p.id;
              return (
                <button
                  key={p.id}
                  onClick={() => setRoutingPolicy(p.id)}
                  className={`flex items-center space-x-1 px-2 py-0.5 rounded-md border transition-all ${
                    isSelected 
                      ? "bg-surface-800 border-cyan-500/50 text-neutral-100 font-semibold glow-cyan" 
                      : "border-transparent text-neutral-500 hover:text-neutral-300 hover:bg-surface-800/40"
                  }`}
                >
                  {p.icon}
                  <span>{p.label}</span>
                </button>
              );
            })}
          </div>

          {/* Quick Slash Shortcuts */}
          <div className="flex items-center space-x-1 text-neutral-500 text-[10px]">
            <span className="cursor-pointer hover:text-cyan-400" onClick={() => setInput("/swarm ")}>/swarm</span>
            <span>•</span>
            <span className="cursor-pointer hover:text-cyan-400" onClick={() => setActiveView("dag")}>/dag</span>
            <span>•</span>
            <span className="cursor-pointer hover:text-cyan-400" onClick={() => setActiveView("diff")}>/diff</span>
            <span>•</span>
            <span className="cursor-pointer hover:text-cyan-400" onClick={() => setActiveView("terminal")}>/terminal</span>
          </div>
        </div>

        {/* Input & Action Form */}
        <form onSubmit={handleSubmit} className="relative flex items-center">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask a question, issue a sandboxed command, or type /swarm <goal>..."
            className="w-full bg-surface-950/80 border border-border-700 focus:border-cyan-500/70 rounded-xl pl-4 pr-32 py-3 text-xs text-neutral-100 placeholder:text-neutral-500 outline-none font-mono transition-all shadow-inner"
          />

          <div className="absolute right-2 flex items-center space-x-1.5">
            <button
              type="button"
              onClick={() => dispatchSwarmTask(input || "Decompose task and execute autonomous loop")}
              className="px-2.5 py-1.5 bg-purple-600/20 hover:bg-purple-600/30 text-purple-300 border border-purple-500/40 rounded-lg text-xs font-mono font-semibold flex items-center space-x-1 transition-all"
              title="Launch Multi-Agent Swarm"
            >
              <Users className="w-3.5 h-3.5" />
              <span>SWARM</span>
            </button>

            <button
              type="submit"
              className="px-3 py-1.5 bg-cyan-500 hover:bg-cyan-400 text-neutral-950 font-bold rounded-lg text-xs font-mono flex items-center space-x-1 transition-all shadow-lg glow-cyan"
            >
              <span>RUN</span>
              <Send className="w-3 h-3" />
            </button>
          </div>
        </form>
      </div>
    </footer>
  );
};
