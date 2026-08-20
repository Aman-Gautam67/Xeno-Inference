//! Real-time telemetry HUD displaying VRAM, GPU load, token velocity, and TTFT.

use xeno_telemetry::metrics::HardwareMetrics;

/// Telemetry HUD model.
#[derive(Debug, Clone, Default)]
pub struct HudState {
    pub hardware: HardwareMetrics,
    pub velocity: f64,
    pub ttft_ms: u64,
    pub estimated_cost: f64,
    pub active_provider: String,
}

impl HudState {
    /// Renders the HUD text widget.
    pub fn render(&self) -> String {
        let vram_used_gb = (self.hardware.vram_allocated_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        let vram_total_gb = (self.hardware.vram_total_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        let vram_pct = ((vram_used_gb / vram_total_gb.max(1.0)) * 100.0) as usize;

        let bar_width = 20usize;
        let filled = (bar_width * vram_pct) / 100;
        let unfilled = bar_width.saturating_sub(filled);
        let vram_bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(unfilled));

        let mut out = String::new();
        out.push_str("┌─ TELEMETRY & HARDWARE HUD ─────────────────────────────┐\n");
        out.push_str(&format!("│ VRAM: {} {:.1}/{:.1} GB ({}%)\n", vram_bar, vram_used_gb, vram_total_gb, vram_pct));
        out.push_str(&format!("│ GPU LOAD: {:.1}% | TTFT: {}ms\n", self.hardware.gpu_core_utilization_pct, self.ttft_ms));
        out.push_str(&format!("│ TOKEN SPEED: {:.1} tok/s | EST COST: ${:.4}\n", self.velocity, self.estimated_cost));
        out.push_str(&format!("│ ACTIVE PROVIDER: {}\n", self.active_provider));
        out.push_str("└────────────────────────────────────────────────────────┘\n");
        out
    }
}
