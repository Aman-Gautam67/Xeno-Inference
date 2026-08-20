//! XENO INFERENCE — High-Performance Terminal User Interface (`xeno-cli`).

pub mod app;
pub mod ui;

use app::XenoCliApp;

#[tokio::main]
async fn main() {
    let app = XenoCliApp::new();
    println!("{}", app.render_frame());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_app_render_frame_and_input() {
        let mut app = XenoCliApp::new();
        let frame = app.render_frame();

        assert!(frame.contains("XENO"));
        assert!(frame.contains("TELEMETRY & HARDWARE HUD"));
        assert!(frame.contains("LIVE EXECUTION DAG"));
        assert!(frame.contains("ACTIVE AST DIFF"));

        let res = app.handle_input("/dag").await;
        assert!(res.contains("Switched to DAG view"));

        let quit_res = app.handle_input("/quit").await;
        assert!(app.should_exit);
        assert!(quit_res.contains("Exiting"));
    }
}
