import React, { useState } from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { 
  BrainCircuit, 
  GitBranch, 
  CheckCircle2, 
  Clock, 
  ChevronDown, 
  ChevronRight, 
  Sparkles, 
  XCircle, 
  Zap,
  Activity
} from "lucide-react";

export const DeepThinkingTimeline: React.FC = () => {
  const { timelineSteps, speculativeBranches } = useWorkspaceStore();
  const [openSteps, setOpenSteps] = useState<Record<string, boolean>>({
    "step-1": true,
    "step-2": true,
    "step-3": true,
  });

  const toggleStep = (id: string) => {
    setOpenSteps((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex bg-void overflow-hidden text-xs font-mono">
      {/* Left: Cognitive Reasoning Stream */}
      <div className="flex-1 p-8 overflow-y-auto canvas-grid-pattern space-y-6">
        <div className="flex items-center justify-between pb-4 border-b border-border-700">
          <div className="flex items-center space-x-2">
            <BrainCircuit className="w-4 h-4 text-purple-400" />
            <h2 className="text-sm font-bold text-neutral-100 tracking-wider">DEEP THINKING & REASONING TIMELINE</h2>
          </div>
          <span className="text-[10px] px-2 py-0.5 rounded bg-purple-500/10 border border-purple-500/30 text-purple-300">
            Observable PAORV Cognitive Stream
          </span>
        </div>

        {/* Cognitive Steps */}
        <div className="space-y-4 max-w-3xl mx-auto">
          {timelineSteps.map((step) => {
            const isOpen = openSteps[step.id];
            return (
              <div
                key={step.id}
                className="rounded-2xl border border-border-700 bg-surface-900/90 backdrop-blur-xl shadow-2xl overflow-hidden transition-all"
              >
                {/* Step Header */}
                <div
                  onClick={() => toggleStep(step.id)}
                  className="p-4 flex items-center justify-between bg-surface-850 cursor-pointer hover:bg-surface-800/80 transition-all"
                >
                  <div className="flex items-center space-x-3">
                    <div className="w-6 h-6 rounded-lg bg-surface-800 border border-border-700 flex items-center justify-center font-bold text-cyan-400 text-xs">
                      {step.stepNumber}
                    </div>
                    <div>
                      <div className="font-bold text-neutral-200 text-xs">{step.title}</div>
                      <div className="text-[10px] text-neutral-500 flex items-center gap-2 mt-0.5">
                        <span className="text-purple-400">{step.phase}</span>
                        <span>•</span>
                        <span>{step.latencyMs}ms elapsed</span>
                        <span>•</span>
                        <span>{step.tokens} tokens ({step.speed} tok/s)</span>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center space-x-3">
                    {step.status === "verified" ? (
                      <span className="flex items-center gap-1 text-[10px] text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded-full border border-emerald-500/30">
                        <CheckCircle2 className="w-3 h-3" /> VERIFIED
                      </span>
                    ) : (
                      <span className="flex items-center gap-1 text-[10px] text-cyan-400 bg-cyan-950/60 px-2 py-0.5 rounded-full border border-cyan-500/30">
                        <Activity className="w-3 h-3 animate-spin" /> EXECUTING
                      </span>
                    )}
                    {isOpen ? <ChevronDown className="w-4 h-4 text-neutral-500" /> : <ChevronRight className="w-4 h-4 text-neutral-500" />}
                  </div>
                </div>

                {/* Step Content */}
                {isOpen && (
                  <div className="p-4 bg-surface-950/70 border-t border-border-700/60 space-y-2">
                    {step.details.map((detail, idx) => (
                      <div key={idx} className="flex items-start space-x-2 text-neutral-300 text-[11px]">
                        <span className="text-cyan-400 font-bold mt-0.5">›</span>
                        <span className="leading-relaxed">{detail}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Right: Speculative Reasoning Tree */}
      <div className="w-96 border-l border-border-700 bg-surface-900/95 p-6 space-y-6 overflow-y-auto">
        <div className="pb-3 border-b border-border-700 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <GitBranch className="w-4 h-4 text-amber-400" />
            <span className="font-bold text-neutral-200 uppercase tracking-wider text-xs">Speculative Branches</span>
          </div>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-surface-800 text-neutral-400 border border-border-700">
            2 Evaluated
          </span>
        </div>

        <div className="space-y-4">
          {speculativeBranches.map((branch) => {
            const isSelected = branch.status === "selected";
            return (
              <div
                key={branch.id}
                className={`p-4 rounded-2xl border bg-surface-950/80 shadow-xl space-y-2.5 transition-all ${
                  isSelected 
                    ? "border-emerald-500/50 glow-emerald" 
                    : "border-border-700 opacity-60 hover:opacity-100"
                }`}
              >
                <div className="flex items-center justify-between">
                  <span className="font-bold text-xs text-neutral-200">{branch.name}</span>
                  {isSelected ? (
                    <span className="text-[10px] px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-500/30 flex items-center gap-1 font-bold">
                      <CheckCircle2 className="w-3 h-3" /> SELECTED
                    </span>
                  ) : (
                    <span className="text-[10px] px-2 py-0.5 rounded bg-rose-950/60 text-rose-400 border border-rose-500/30 flex items-center gap-1 font-bold">
                      <XCircle className="w-3 h-3" /> PRUNED
                    </span>
                  )}
                </div>

                <p className="text-[11px] text-neutral-400 leading-relaxed">
                  {branch.rationale}
                </p>

                <div className="pt-2 border-t border-border-700/60 flex items-center justify-between text-[10px] text-neutral-500">
                  <span>Confidence: <strong className={isSelected ? "text-emerald-400" : "text-neutral-400"}>{branch.score}%</strong></span>
                  <span>Latency: ~{branch.latencyEstimateMs}ms</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
