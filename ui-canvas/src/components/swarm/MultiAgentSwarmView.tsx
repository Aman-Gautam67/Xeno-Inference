import React from "react";
import { useWorkspaceStore } from "../../stores/workspaceStore";
import { 
  Users, 
  Crown, 
  Box, 
  Code2, 
  FlaskConical, 
  ShieldAlert, 
  CheckCircle2, 
  Activity, 
  MessageSquare,
  Sparkles,
  Zap
} from "lucide-react";

export const MultiAgentSwarmView: React.FC = () => {
  const { swarmAgents, consensusRate } = useWorkspaceStore();

  const getRoleIcon = (role: string) => {
    switch (role) {
      case "commander": return <Crown className="w-5 h-5 text-stone-700 dark:text-stone-300" />;
      case "architect": return <Box className="w-5 h-5 text-purple-600 dark:text-purple-400" />;
      case "coder": return <Code2 className="w-5 h-5 text-stone-900 dark:text-stone-100" />;
      case "qa": return <FlaskConical className="w-5 h-5 text-amber-600 dark:text-amber-400" />;
      case "red_team": return <ShieldAlert className="w-5 h-5 text-rose-600 dark:text-rose-400" />;
      default: return <Users className="w-5 h-5 text-stone-500" />;
    }
  };

  const swarmMessages = [
    { sender: "Commander", time: "18:44:10", text: "Goal initialized: Decompose AST validation into subtasks." },
    { sender: "Architect", time: "18:44:12", text: "Verified schema compatibility across all 8 workspace crates." },
    { sender: "Coder", time: "18:44:15", text: "Synthesized atomic patch for ast_validator.rs with character bounds." },
    { sender: "QA Tester", time: "18:44:18", text: "Executed 120+ cargo tests against modified crate. All passing." },
    { sender: "Red-Team", time: "18:44:20", text: "Fuzz test complete. Air-gap socket verified with 0 secret leaks." },
  ];

  return (
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex flex-col bg-stone-50 dark:bg-stone-950 text-xs font-mono overflow-y-auto p-8 space-y-6 transition-colors duration-200">
      {/* Top Banner */}
      <div className="flex items-center justify-between pb-4 border-b border-stone-200 dark:border-stone-800">
        <div className="flex items-center space-x-3">
          <div className="p-2.5 rounded-2xl bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-800 shadow-sm text-stone-700 dark:text-stone-300">
            <Users className="w-5 h-5" />
          </div>
          <div>
            <h2 className="font-display font-bold text-base text-stone-900 dark:text-stone-100 tracking-wide uppercase">
              Autonomous Swarm Council
            </h2>
            <div className="text-[11px] text-stone-500 font-sans">5-Role Multi-Agent Consensus & Deliberation Matrix</div>
          </div>
        </div>

        <div className="flex items-center space-x-4 bg-white dark:bg-stone-900 p-3 rounded-2xl border border-stone-200 dark:border-stone-800 shadow-sm">
          <div>
            <div className="text-[9px] text-stone-400 dark:text-stone-500 uppercase font-bold">Consensus Rate</div>
            <div className="text-emerald-700 dark:text-emerald-400 font-bold text-sm">{consensusRate}% PASS</div>
          </div>
          <div className="w-24 h-2 bg-stone-100 dark:bg-stone-800 rounded-full overflow-hidden">
            <div className="h-full bg-emerald-600 dark:bg-emerald-500 rounded-full" style={{ width: `${consensusRate}%` }} />
          </div>
        </div>
      </div>

      {/* 5-Role Swarm Council Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
        {swarmAgents.map((agent) => (
          <div
            key={agent.role}
            className="p-4 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-sm card-elevation space-y-3 flex flex-col justify-between"
          >
            <div className="space-y-2">
              <div className="flex items-center justify-between pb-2 border-b border-stone-100 dark:border-stone-800">
                <div className="p-2 rounded-xl bg-stone-50 dark:bg-stone-800">
                  {getRoleIcon(agent.role)}
                </div>
                <span className="text-[9px] font-mono px-2 py-0.5 rounded-full bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 font-bold">
                  VOTE {agent.voteScore}%
                </span>
              </div>

              <div>
                <h3 className="font-display font-bold text-xs text-stone-900 dark:text-stone-100">{agent.title}</h3>
                <div className="text-[10px] text-stone-500">{agent.model}</div>
              </div>

              <p className="text-[11px] text-stone-600 dark:text-stone-400 font-sans leading-tight bg-stone-50 dark:bg-stone-950 p-2.5 rounded-xl border border-stone-200 dark:border-stone-800">
                {agent.currentTask}
              </p>
            </div>

            <div className="flex items-center justify-between text-[10px] text-stone-500 pt-2 border-t border-stone-100 dark:border-stone-800 font-mono">
              <span>{agent.tokensGenerated.toLocaleString()} tok</span>
              <span className="text-emerald-600 dark:text-emerald-400 font-semibold uppercase">{agent.status}</span>
            </div>
          </div>
        ))}
      </div>

      {/* Swarm Communication Stream */}
      <div className="p-5 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-sm space-y-3">
        <div className="flex items-center justify-between pb-2 border-b border-stone-100 dark:border-stone-800">
          <div className="flex items-center space-x-2">
            <MessageSquare className="w-4 h-4 text-stone-500" />
            <span className="font-display font-bold text-xs uppercase tracking-wider text-stone-800 dark:text-stone-200">
              Inter-Agent Consensus Communication Bus
            </span>
          </div>
          <span className="text-[10px] text-emerald-600 dark:text-emerald-400 font-bold">3-Way Quorum Verified</span>
        </div>

        <div className="space-y-2">
          {swarmMessages.map((msg, i) => (
            <div key={i} className="flex items-center space-x-3 text-xs leading-relaxed p-2 rounded-xl bg-stone-50/70 dark:bg-stone-950 border border-stone-200/60 dark:border-stone-800/60">
              <span className="text-stone-400 font-mono text-[10px]">{msg.time}</span>
              <span className="font-bold font-display text-stone-900 dark:text-stone-100 min-w-[90px]">{msg.sender}:</span>
              <span className="text-stone-700 dark:text-stone-300 font-sans">{msg.text}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
