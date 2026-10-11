// src/main.rs

mod domain;
mod ports;
mod orchestrator;

use ports::{TextCommandSource, MockMediaRenderer};
use orchestrator::Orchestrator;

fn main() {
    // 1. Simulating commands pasted or read from commands.txt
    let raw_commands = r#"
        # Video cut plan
        DROP 10.0 - 25.0
        DROP 70.0 - 90.0
    "#;

    // 2. Instantiate our plug & play modules
    let command_source = TextCommandSource {
        raw_content: raw_commands.to_string(),
    };
    let media_renderer = MockMediaRenderer;

    // 3. Assemble Orchestrator
    let app = Orchestrator::new(command_source, media_renderer);

    // 4. Run pipeline on a 120-second video
    let video_duration = 120.0;
    println!("=== Testing CutEngine CLI ===");
    
    if let Err(err) = app.run_pipeline("sample_input.mkv", "sample_output.mkv", video_duration) {
        eprintln!("Pipeline execution error: {}", err);
    }
}