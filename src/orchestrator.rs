// src/orchestrator.rs
use crate::domain::Timeline;
use crate::ports::{Command, CommandSource, MediaRenderer};

pub struct Orchestrator<C: CommandSource, M: MediaRenderer> {
    command_source: C,
    media_renderer: M,
}

impl<C: CommandSource, M: MediaRenderer> Orchestrator<C, M> {
    /// Creates a new Orchestrator by plugging in the components.
    pub fn new(command_source: C, media_renderer: M) -> Self {
        Self {
            command_source,
            media_renderer,
        }
    }

    /// Executes the full rendering pipeline.
    pub fn run_pipeline(&self, input_file: &str, output_file: &str, video_duration: f32) -> Result<(), &'static str> {
        println!("Starting CutEngine pipeline...");

        // 1. Initialize mathematical timeline
        let mut timeline = Timeline::new(video_duration);

        // 2. Fetch commands via Plug & Play source (File, STDIN, Mock, etc.)
        let commands = self.command_source.fetch_commands()?;

        // 3. Apply commands to timeline
        for cmd in commands {
            match cmd {
                Command::Drop(start, end) => timeline.apply_drop(start, end),
                Command::Keep(_start, _end) => {
                    // Logic for Keep can be implemented similarly if additive features are needed
                }
            }
        }

        // 4. Send the normalized final timeline to the Plug & Play renderer (FFmpeg or Mock)
        self.media_renderer.render_copy(input_file, &timeline.segments, output_file)?;

        println!("Pipeline finished successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{MockCommandSource, MockMediaRenderer};

    #[test]
    fn test_orchestrator_mock_pipeline() {
        // Plug in our Mocks
        let mock_source = MockCommandSource {
            predefined_commands: vec![Command::Drop(10.0, 20.0)],
        };
        let mock_renderer = MockMediaRenderer;

        // Initialize Orchestrator
        let app = Orchestrator::new(mock_source, mock_renderer);

        // Run the pipeline on a dummy 60-second video
        let result = app.run_pipeline("raw_video.mkv", "final_cut.mkv", 60.0);
        
        assert!(result.is_ok());
    }
}
