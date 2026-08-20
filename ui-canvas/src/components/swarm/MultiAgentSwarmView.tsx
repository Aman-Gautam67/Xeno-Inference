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
      case "commander": return <Crown className="w-5 h-5 text-rose-400" />;
      case "architect": return <Box className="w-5 h-5 text-purple-400" />;
      case "coder": return <Code2 className="w-5 h-5 text-cyan-400" />;
      case "qa": return <FlaskConical className="w-5 h-5 text-amber-400" />;
      case "red_team": return <ShieldAlert className="w-5 h-5 text-crimson-400" />;
      default: return <Users className="w-5 h-5 text-neutral-400" />;
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
    <div className="flex-1 h-[calc(100vh-3.5rem)] flex flex-col bg-void text-xs font-mono overflow-y-auto p-8 space-y-6">
      {/* Top Banner */}
      <div className="flex items-center justify-between pb-4 border-b border-border-700">
        <div className="flex items-center space-x-3">
          <div className="p-2 rounded-xl bg-purple-500/10 border border-purple-500/30 text-purple-400">
            <Users className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-sm font-bold text-neutral-100 tracking-wider">MULTI-AGENT SWARM ARENA</h2>
            <div className="text-[10px] text-neutral-500">Autonomous 5-Role Council • 3-Way Cross-Consensus Engine</div>
          </div>
        </div>

        <div className="flex items-center space-x-4 bg-surface-900/80 p-3 rounded-2xl border border-border-700">
          <div>
            <div className="text-[9px] text-neutral-500 uppercase">Cross-Consensus</div>
            <div className="text-emerald-400 font-bold text-sm">{consensusRate}% PASS</div>
          </div>
          <div className="w-20 h-2 bg-surface-800 rounded-full overflow-hidden">
            <div className="h-full bg-emerald-400" style={{ width: `${consensusRate}%` }} />
          </div>
        </div>
      </div>

      {/* Swarm Council Cards */}
      <div className="grid grid-cols-5 gap-4">
        {swarmAgents.map((agent) => (
          <div
            key={agent.role}
            className="p-4 rounded-2xl border border-border-700 bg-surface-900/90 backdrop-blur-xl shadow-xl space-y-3 relative overflow-hidden"
          >
            <div className="flex items-center justify-between">
              <div className="p-2 rounded-xl bg-surface-800 border border-border-700">
                {getRoleIcon(agent.role)}
              </div>
              <span className={`text-[9px] px-2 py-0.5 rounded-full uppercase font-bold border ${
                agent.status === "coding"
                  ? "bg-cyan-500/20 text-cyan-300 border-cyan-500/40 glow-cyan"
                  : agent.status === "planning"
                  ? "bg-purple-500/20 text-purple-300 border-purple-500/40"
                  : "bg-surface-800 text-neutral-400 border-border-700"
              }`}>
                {agent.status}
              </span>
            </div>

            <div>
              <div className="font-bold text-neutral-200 text-xs">{agent.title}</div>
              <div className="text-[10px] text-neutral-500 truncate">{agent.model}</div>
            </div>

            <div className="text-[11px] text-neutral-300 bg-surface-950 p-2.5 rounded-xl border border-border-700/60 leading-relaxed min-h-[48px]">
              {agent.currentTask}
            </div>

            <div className="pt-2 border-t border-border-700 flex items-center justify-between text-[10px] text-neutral-500">
              <span className="flex items-center gap-1">
                <Zap className="w-3 h-3 text-cyan-400" />
                {agent.tokensGenerated} tok
              </span>
              <span className="text-emerald-400 font-bold">Vote: {agent.voteScore}%</span>
            </div>
          </div>
        ))}
      </div>

      {/* Inter-Agent Message Log */}
      <div className="space-y-3 pt-4">
        <div className="flex items-center space-x-2 text-neutral-400">
          <MessageSquare className="w-4 h-4 text-cyan-400" />
          <span className="font-bold text-xs uppercase tracking-wider">Swarm Inter-Agent Cognitive Channel</span>
        </div>

        <div className="p-4 rounded-2xl border border-border-700 bg-surface-950 space-y-2.5">
          {swarmMessages.map((msg, i) => (
            <div key={i} className="flex items-start space-x-3 text-neutral-300 text-[11px]">
              <span className="text-neutral-600">[{msg.time}]</span>
              <span className="font-bold text-cyan-400 uppercase w-24 truncate">{msg.sender}:</span>
              <span className="leading-relaxed flex-1">{msg.text}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
