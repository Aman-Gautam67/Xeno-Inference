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
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex bg-stone-50 dark:bg-stone-950 overflow-hidden text-xs font-mono transition-colors duration-200">
      {/* Left: Cognitive Reasoning Stream */}
      <div className="flex-1 p-8 overflow-y-auto canvas-grid-pattern space-y-6">
        <div className="flex items-center justify-between pb-4 border-b border-stone-200 dark:border-stone-800">
          <div className="flex items-center space-x-2">
            <BrainCircuit className="w-4 h-4 text-purple-600 dark:text-purple-400" />
            <h2 className="font-display font-bold text-sm tracking-wider uppercase text-stone-900 dark:text-stone-100">
              Deep Thinking & Cognitive Timeline
            </h2>
          </div>
          <span className="text-[10px] px-2 py-0.5 rounded-full bg-purple-50 dark:bg-purple-950/60 border border-purple-200 dark:border-purple-800 text-purple-700 dark:text-purple-300 font-bold">
            PAORV Observable Stream
          </span>
        </div>

        {/* Cognitive Steps */}
        <div className="space-y-4 max-w-3xl mx-auto">
          {timelineSteps.map((step) => {
            const isOpen = openSteps[step.id];
            return (
              <div
                key={step.id}
                className="rounded-2xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-sm card-elevation overflow-hidden transition-all"
              >
                {/* Step Header */}
                <div
                  onClick={() => toggleStep(step.id)}
                  className="p-4 flex items-center justify-between bg-stone-50 dark:bg-stone-850 cursor-pointer hover:bg-stone-100 dark:hover:bg-stone-800 transition-all border-b border-stone-100 dark:border-stone-800"
                >
                  <div className="flex items-center space-x-3">
                    <div className="w-6 h-6 rounded-lg bg-white dark:bg-stone-800 border border-stone-200 dark:border-stone-700 flex items-center justify-center font-bold text-stone-800 dark:text-stone-200 text-xs shadow-sm">
                      {step.stepNumber}
                    </div>
                    <div>
                      <div className="font-bold text-stone-900 dark:text-stone-100 text-xs font-display">{step.title}</div>
                      <div className="text-[10px] text-stone-500 uppercase font-semibold">{step.phase}</div>
                    </div>
                  </div>

                  <div className="flex items-center space-x-3">
                    <div className="text-[10px] text-stone-500 flex items-center space-x-2">
                      <span>{step.latencyMs}ms</span>
                      <span>•</span>
                      <span>{step.tokens} tok</span>
                    </div>

                    {step.status === "verified" ? (
                      <span className="flex items-center gap-1 text-[10px] text-emerald-700 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-950/60 px-2 py-0.5 rounded-full border border-emerald-200 dark:border-emerald-800 font-bold">
                        <CheckCircle2 className="w-3 h-3 text-emerald-600" /> VERIFIED
                      </span>
                    ) : (
                      <span className="flex items-center gap-1 text-[10px] text-amber-700 dark:text-amber-300 bg-amber-50 dark:bg-amber-950/60 px-2 py-0.5 rounded-full border border-amber-200 dark:border-amber-800 font-bold">
                        <Activity className="w-3 h-3 animate-spin text-amber-600" /> EXECUTING
                      </span>
                    )}

                    {isOpen ? <ChevronDown className="w-4 h-4 text-stone-400" /> : <ChevronRight className="w-4 h-4 text-stone-400" />}
                  </div>
                </div>

                {/* Collapsible Details */}
                {isOpen && (
                  <div className="p-4 bg-stone-50/50 dark:bg-stone-950 space-y-2">
                    {step.details.map((detail, idx) => (
                      <div key={idx} className="flex items-start space-x-2 text-stone-700 dark:text-stone-300 leading-relaxed font-sans text-xs">
                        <span className="text-stone-400 mt-0.5">•</span>
                        <span>{detail}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Right: Speculative Branch Evaluator */}
      <div className="w-80 border-l border-stone-200 dark:border-stone-800 bg-white/95 dark:bg-stone-900/95 p-5 space-y-4 overflow-y-auto">
        <div className="flex items-center justify-between pb-3 border-b border-stone-200 dark:border-stone-800">
          <div className="flex items-center space-x-2">
            <GitBranch className="w-4 h-4 text-amber-600 dark:text-amber-400" />
            <span className="font-display font-bold text-xs uppercase tracking-wider text-stone-800 dark:text-stone-200">
              Speculative Branches
            </span>
          </div>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 font-bold">
            2 Hypotheses
          </span>
        </div>

        <div className="space-y-3">
          {speculativeBranches.map((b) => (
            <div
              key={b.id}
              className={`p-3.5 rounded-xl border space-y-2 transition-all ${
                b.status === "selected"
                  ? "bg-emerald-50/50 dark:bg-emerald-950/20 border-emerald-300 dark:border-emerald-800/80 shadow-sm"
                  : "bg-stone-50 dark:bg-stone-950 border-stone-200 dark:border-stone-800 opacity-70"
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="font-bold text-stone-900 dark:text-stone-100 font-display text-xs">{b.name}</span>
                {b.status === "selected" ? (
                  <span className="text-[9px] px-1.5 py-0.2 rounded bg-emerald-100 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 font-bold">
                    SELECTED (96%)
                  </span>
                ) : (
                  <span className="text-[9px] px-1.5 py-0.2 rounded bg-stone-200 dark:bg-stone-800 text-stone-500 font-bold">
                    PRUNED (64%)
                  </span>
                )}
              </div>

              <p className="text-[11px] text-stone-600 dark:text-stone-400 font-sans leading-relaxed">
                {b.rationale}
              </p>

              <div className="flex items-center justify-between text-[10px] text-stone-400 pt-1 border-t border-stone-200 dark:border-stone-800">
                <span>Est. Latency: {b.latencyEstimateMs}ms</span>
                <span className="text-emerald-600 dark:text-emerald-400 font-bold">Confidence: {b.score}%</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
