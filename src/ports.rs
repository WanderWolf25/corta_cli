// src/ports.rs
use crate::domain::TimeRange;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Drop(f32, f32),
    #[allow(dead_code)]
    Keep(f32, f32),
}

/// Parses formats: "HH:MM:SS.mmm", "MM:SS.mmm", or "SS.mmm" into seconds (f32).
pub fn parse_timestamp(raw: &str) -> Result<f32, &'static str> {
    let clean = raw.trim();
    let parts: Vec<&str> = clean.split(':').collect();

    match parts.len() {
        // Formato decimal directo: "83.5"
        1 => parts[0]
            .parse::<f32>()
            .map_err(|_| "Failed to parse seconds"),

        // Formato "MM:SS.mmm": "01:23.500"
        2 => {
            let minutes = parts[0]
                .parse::<f32>()
                .map_err(|_| "Failed to parse minutes")?;
            let seconds = parts[1]
                .parse::<f32>()
                .map_err(|_| "Failed to parse seconds")?;
            Ok(minutes * 60.0 + seconds)
        }

        // Formato "HH:MM:SS.mmm": "00:01:23.500"
        3 => {
            let hours = parts[0]
                .parse::<f32>()
                .map_err(|_| "Failed to parse hours")?;
            let minutes = parts[1]
                .parse::<f32>()
                .map_err(|_| "Failed to parse minutes")?;
            let seconds = parts[2]
                .parse::<f32>()
                .map_err(|_| "Failed to parse seconds")?;
            Ok(hours * 3600.0 + minutes * 60.0 + seconds)
        }

        _ => Err("Invalid timestamp format"),
    }
}

// ==========================================
// TRAITS (Plug & Play Contracts)
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
// IMPLEMENTACIONES CONCRETAS Y MOCKS
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
                let start = parse_timestamp(parts[1])?;
                let end = parse_timestamp(parts[3])?;
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
            println!("  -> Concatenating chunk {}: [{:.3}s - {:.3}s]", i, range.start, range.end);
        }
        println!("[MOCK FFMPEG] Successfully rendered to: {}", output_file);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp_formats() {
        assert_eq!(parse_timestamp("15.5").unwrap(), 15.5);
        assert_eq!(parse_timestamp("01:30.0").unwrap(), 90.0);
        assert_eq!(parse_timestamp("01:00:00.0").unwrap(), 3600.0);
        assert_eq!(parse_timestamp("00:02:15.500").unwrap(), 135.5);
    }
}