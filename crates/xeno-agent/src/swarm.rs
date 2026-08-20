//! Hierarchical Swarm roles council and 3-way consensus verification engine.

use serde::{Deserialize, Serialize};

/// Specialized roles in the autonomous swarm council.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmRole {
    Commander,
    Architect,
    Coder,
    QATester,
    RedTeamAuditor,
}

impl Default for SwarmRole {
    fn default() -> Self {
        SwarmRole::Commander
    }
}

impl SwarmRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Commander => "commander",
            Self::Architect => "architect",
            Self::Coder => "coder",
            Self::QATester => "qa_tester",
            Self::RedTeamAuditor => "red_team",
        }
    }
}

/// An individual audit vote from a model or role evaluator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsensusVote {
    pub voter_id: String,
    pub role: SwarmRole,
    pub model_name: String,
    pub approved: bool,
    pub confidence: f64,
    pub findings: Vec<String>,
}

/// Result of a 3-way cross-model consensus evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsensusEvaluation {
    pub approved: bool,
    pub total_votes: usize,
    pub positive_votes: usize,
    pub consensus_ratio: f64,
    pub votes: Vec<ConsensusVote>,
    pub decision_summary: String,
}

/// Cross-model and multi-agent consensus evaluator.
#[derive(Debug, Clone, Default)]
pub struct ConsensusChecker;

impl ConsensusChecker {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates consensus across a set of votes with a threshold (default: 0.66).
    pub fn evaluate_consensus(
        &self,
        votes: Vec<ConsensusVote>,
        threshold: Option<f64>,
    ) -> ConsensusEvaluation {
        let thresh = threshold.unwrap_or(0.66);
        let total = votes.len();
        if total == 0 {
            return ConsensusEvaluation {
                approved: false,
                total_votes: 0,
                positive_votes: 0,
                consensus_ratio: 0.0,
                votes: Vec::new(),
                decision_summary: "No votes submitted".into(),
            };
        }

        let positive = votes.iter().filter(|v| v.approved).count();
        let ratio = (positive as f64) / (total as f64);
        let approved = ratio >= thresh;

        let summary = if approved {
            format!("Consensus reached with {positive}/{total} votes ({:.1}%)", ratio * 100.0)
        } else {
            format!("Consensus REJECTED with {positive}/{total} votes ({:.1}%), required {:.1}%", ratio * 100.0, thresh * 100.0)
        };

        ConsensusEvaluation {
            approved,
            total_votes: total,
            positive_votes: positive,
            consensus_ratio: ratio,
            votes,
            decision_summary: summary,
        }
    }
}

/// Multi-agent swarm council orchestrating specialized agents.
#[derive(Debug, Clone, Default)]
pub struct SwarmCouncil {
    consensus_checker: ConsensusChecker,
}

impl SwarmCouncil {
    pub fn new() -> Self {
        Self {
            consensus_checker: ConsensusChecker::new(),
        }
    }

    /// Verifies critical modifications via 3-way consensus.
    pub fn verify_critical_decision(
        &self,
        votes: Vec<ConsensusVote>,
    ) -> ConsensusEvaluation {
        self.consensus_checker.evaluate_consensus(votes, Some(0.66))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_evaluation() {
        let checker = ConsensusChecker::new();
        let votes = vec![
            ConsensusVote {
                voter_id: "agent_1".into(),
                role: SwarmRole::Architect,
                model_name: "claude-3-7-sonnet".into(),
                approved: true,
                confidence: 0.95,
                findings: vec![],
            },
            ConsensusVote {
                voter_id: "agent_2".into(),
                role: SwarmRole::QATester,
                model_name: "deepseek-reasoner".into(),
                approved: true,
                confidence: 0.90,
                findings: vec![],
            },
            ConsensusVote {
                voter_id: "agent_3".into(),
                role: SwarmRole::RedTeamAuditor,
                model_name: "qwen-2.5-72b".into(),
                approved: false,
                confidence: 0.80,
                findings: vec!["Minor style issue".into()],
            },
        ];

        let eval = checker.evaluate_consensus(votes, Some(0.66));
        assert!(eval.approved);
        assert_eq!(eval.positive_votes, 2);
        assert_eq!(eval.total_votes, 3);
    }
}
