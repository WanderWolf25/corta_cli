// src/ports.rs
use crate::domain::TimeRange;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Drop(f32, f32),
    #[allow(dead_code)]
    Keep(f32, f32),
}

// ==========================================
// TRAITS (The Plug & Play Contracts)
// ==========================================

pub trait CommandSource {
    fn fetch_commands(&self) -> Result<Vec<Command>, &'static str>;
}

pub trait MediaRenderer {
    fn render_copy(&self, input_file: &str, ranges: &[TimeRange], output_file: &str) -> Result<(), &'static str>;
}

#[allow(dead_code)]
pub trait AiCompiler {
    fn compile_plan(&self, transcript_json: &str, prompt: &str) -> Result<Vec<Command>, &'static str>;
}

// ==========================================
// SOURCES & MOCKS
// ==========================================

pub struct TextCommandSource {
    pub raw_content: String,
}

impl CommandSource for TextCommandSource {
    fn fetch_commands(&self) -> Result<Vec<Command>, &'static str> {
        let mut commands = Vec::new();

        for line in self.raw_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() == 4 && parts[0] == "DROP" && parts[2] == "-" {
                let start = parts[1].parse::<f32>().map_err(|_| "Failed to parse start timestamp")?;
                let end = parts[3].parse::<f32>().map_err(|_| "Failed to parse end timestamp")?;
                commands.push(Command::Drop(start, end));
            } else {
                return Err("Syntax error: line must follow 'DROP <start> - <end>'");
            }
        }

        Ok(commands)
    }
}

#[allow(dead_code)]
pub struct MockCommandSource {
    pub predefined_commands: Vec<Command>,
}

impl CommandSource for MockCommandSource {
    fn fetch_commands(&self) -> Result<Vec<Command>, &'static str> {
        println!("[MOCK] Fetching predefined commands...");
        Ok(self.predefined_commands.clone())
    }
}

pub struct MockMediaRenderer;

impl MediaRenderer for MockMediaRenderer {
    fn render_copy(&self, input_file: &str, ranges: &[TimeRange], output_file: &str) -> Result<(), &'static str> {
        println!("[MOCK FFMPEG] Processing input: {}", input_file);
        for (i, range) in ranges.iter().enumerate() {
            println!("  -> Concatenating chunk {}: [{}s - {}s]", i, range.start, range.end);
        }
        println!("[MOCK FFMPEG] Successfully rendered to: {}", output_file);
        Ok(())
    }
}