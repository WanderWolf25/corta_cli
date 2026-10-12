// src/media.rs
use std::fs::File;
use std::io::Write;
use std::process::Command as SysCommand;
use crate::domain::TimeRange;
use crate::ports::MediaRenderer;

pub struct FfmpegEngine;

impl FfmpegEngine {
    /// Queries the container duration in seconds using ffprobe.
    pub fn probe_duration(input_file: &str) -> Result<f32, &'static str> {
        let output = SysCommand::new("ffprobe")
            .args([
                "-v", "error",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
                input_file,
            ])
            .output()
            .map_err(|_| "Failed to execute ffprobe. Ensure ffprobe is in your system PATH")?;

        if !output.status.success() {
            return Err("ffprobe returned a non-zero exit code");
        }

        let raw_str = String::from_utf8_lossy(&output.stdout);
        raw_str
            .trim()
            .parse::<f32>()
            .map_err(|_| "Failed to parse duration from ffprobe output")
    }
}

impl MediaRenderer for FfmpegEngine {
    fn render_copy(&self, input_file: &str, ranges: &[TimeRange], output_file: &str) -> Result<(), &'static str> {
        if ranges.is_empty() {
            return Err("Cannot render empty timeline");
        }

        let temp_list_path = "concat_list.txt";
        let mut list_file = File::create(temp_list_path)
            .map_err(|_| "Failed to create temporary concat list file")?;

        // Format for ffmpeg concat demuxer:
        // file 'input.mkv'
        // inpoint 0.0
        // outpoint 10.5
        for range in ranges {
            writeln!(list_file, "file '{}'", input_file)
                .map_err(|_| "Failed to write input file to concat list")?;
            writeln!(list_file, "inpoint {:.3}", range.start)
                .map_err(|_| "Failed to write inpoint to concat list")?;
            writeln!(list_file, "outpoint {:.3}", range.end)
                .map_err(|_| "Failed to write outpoint to concat list")?;
        }

        // Flush and close the temporary manifest
        drop(list_file);

        println!("[FFMPEG] Running stream copy via concat demuxer...");
        let status = SysCommand::new("ffmpeg")
            .args([
                "-y",                   // Overwrite output file
                "-f", "concat",
                "-safe", "0",
                "-i", temp_list_path,
                "-c", "copy",
                output_file,
            ])
            .status()
            .map_err(|_| "Failed to execute ffmpeg. Ensure ffmpeg is in your system PATH")?;

        // Cleanup temporary manifest file
        let _ = std::fs::remove_file(temp_list_path);

        if !status.success() {
            return Err("ffmpeg execution failed during stream copy");
        }

        println!("[FFMPEG] Stream copy finished successfully: {}", output_file);
        Ok(())
    }
}
