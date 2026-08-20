//! XENO INFERENCE — Observable Telemetry & Privacy Shield (`xeno-telemetry`).
//!
//! Provides ring-buffer telemetry logging, token velocity tracking, financial cost accounting,
//! hardware metrics sampling, and strict Chain-of-Thought privacy protection.

pub mod collector;
pub mod metrics;
pub mod privacy_guard;

pub use collector::TelemetryCollector;
pub use metrics::{HardwareMetrics, SessionSummaryMetrics, StepTelemetry};
pub use privacy_guard::TelemetryPrivacyGuard;

/// Prelude exporting all telemetry primitives.
pub mod prelude {
    pub use super::collector::TelemetryCollector;
    pub use super::metrics::{HardwareMetrics, SessionSummaryMetrics, StepTelemetry};
    pub use super::privacy_guard::TelemetryPrivacyGuard;
}
