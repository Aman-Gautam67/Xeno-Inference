//! Cyberpunk ASCII banner and system status header renderer.

pub const ASCII_BANNER: &str = r#"
  ██╗  ██╗ ███████╗ ███╗   ██╗  ██████╗     ██╗ ███╗   ██╗ ███████╗ ███████╗ ██████╗  ███████╗ ███╗   ██╗
  ╚██╗██╔╝ ██╔════╝ ████╗  ██║ ██╔═══██╗    ██║ ████╗  ██║ ██╔════╝ ██╔════╝ ██╔══██╗ ██╔════╝ ████╗  ██║
   ╚███╔╝  █████╗   ██╔██╗ ██║ ██║   ██║    ██║ ██╔██╗ ██║ █████╗   █████╗   ██████╔╝ █████╗   ██╔██╗ ██║
   ██╔██╗  ██╔══╝   ██║╚██╗██║ ██║   ██║    ██║ ██║╚██╗██║ ██╔══╝   ██╔══╝   ██╔══██╗ ██╔══╝   ██║╚██╗██║
  ██╔╝ ██╗ ███████╗ ██║ ╚████║ ╚██████╔╝    ██║ ██║ ╚████║ ██║      ███████╗ ██║  ██║ ███████╗ ██║ ╚████║
"#;

/// Renders the formatted terminal header string.
pub fn render_header(mode: &str, provider: &str, velocity: f64, cost: f64) -> String {
    let mut header = String::new();
    header.push_str(ASCII_BANNER);
    header.push_str(&format!(
        " [MODE: {}] | [PROVIDER: {}] | [VELOCITY: {:.1} tok/s] | [COST: ${:.4}]\n",
        mode.to_uppercase(),
        provider,
        velocity,
        cost
    ));
    header.push_str("─────────────────────────────────────────────────────────────────────────────────────────────────────────────\n");
    header
}
