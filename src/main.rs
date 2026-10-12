// src/main.rs

mod domain;
mod media;
mod orchestrator;
mod ports;

use media::FfmpegEngine;
use orchestrator::Orchestrator;
use ports::TextCommandSource;
use std::fs;

fn main() {
    let commands_file = "commands.txt";
    let input_video = "sample_input.mp4";
    let output_video = "sample_output.mp4";

    println!("=== CutEngine CLI ===");

    // 1. Read commands from commands.txt
    let raw_commands = match fs::read_to_string(commands_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading {}: {}", commands_file, err);
            return;
        }
    };

    // 2. Extract real duration via ffprobe
    let video_duration = match FfmpegEngine::probe_duration(input_video) {
        Ok(duration) => {
            println!("Video duration: {:.2}s", duration);
            duration
        }
        Err(err) => {
            eprintln!("Error probing video: {}", err);
            return;
        }
    };

    // 3. Assemble and execute pipeline
    let command_source = TextCommandSource {
        raw_content: raw_commands,
    };
    let media_renderer = FfmpegEngine;
    let app = Orchestrator::new(command_source, media_renderer);

    if let Err(err) = app.run_pipeline(input_video, output_video, video_duration) {
        eprintln!("Pipeline execution error: {}", err);
    }
}
