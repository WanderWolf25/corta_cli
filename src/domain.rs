// src/domain.rs

#[derive(Debug, Clone, PartialEq)]
pub struct TimeRange {
    pub start: f32,
    pub end: f32,
}

impl TimeRange {
    /// Creates a new TimeRange, validating that start is before end.
    pub fn new(start: f32, end: f32) -> Result<Self, &'static str> {
        if start < 0.0 || start >= end {
            return Err("Invalid time range: start must be >= 0 and < end");
        }
        Ok(Self { start, end })
    }
}

pub struct Timeline {
    pub total_duration: f32,
    pub segments: Vec<TimeRange>,
}

impl Timeline {
    /// Initializes a subtractive timeline assuming the whole video is kept initially.
    pub fn new(total_duration: f32) -> Self {
        let initial_segment = TimeRange::new(0.0, total_duration)
            .expect("Initial segment duration must be positive");

        Self {
            total_duration,
            segments: vec![initial_segment],
        }
    }

    /// Subtracts a time range, splitting or deleting existing segments as necessary.
    pub fn apply_drop(&mut self, mut drop_start: f32, mut drop_end: f32) {
        if drop_start >= drop_end || drop_start >= self.total_duration {
            return;
        }

        // Clamp drop boundaries to video limits
        drop_start = drop_start.max(0.0);
        drop_end = drop_end.min(self.total_duration);

        let mut new_segments = Vec::new();

        for seg in &self.segments {
            if drop_end <= seg.start || drop_start >= seg.end {
                new_segments.push(seg.clone());
            } else if drop_start <= seg.start && drop_end >= seg.end {
                continue;
            } else if drop_start > seg.start && drop_end < seg.end {
                if let (Ok(left), Ok(right)) = (
                    TimeRange::new(seg.start, drop_start),
                    TimeRange::new(drop_end, seg.end),
                ) {
                    new_segments.push(left);
                    new_segments.push(right);
                }
            } else if drop_start <= seg.start && drop_end < seg.end {
                if let Ok(right) = TimeRange::new(drop_end, seg.end) {
                    new_segments.push(right);
                }
            } else if drop_start > seg.start && drop_end >= seg.end {
                if let Ok(left) = TimeRange::new(seg.start, drop_start) {
                    new_segments.push(left);
                }
            }
        }

        self.segments = new_segments;
        self.normalize();
    }

    /// Sorts the segments and merges those that are very close (epsilon).
    pub fn normalize(&mut self) {
        if self.segments.is_empty() {
            return;
        }

        self.segments.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

        let mut merged = Vec::new();
        let mut current = self.segments[0].clone();
        let epsilon = 0.05;

        for next in self.segments.iter().skip(1) {
            if next.start <= current.end + epsilon {
                current.end = current.end.max(next.end);
            } else {
                merged.push(current);
                current = next.clone();
            }
        }
        merged.push(current);
        self.segments = merged;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_validation() {
        assert!(TimeRange::new(10.0, 20.0).is_ok());
        assert!(TimeRange::new(20.0, 10.0).is_err());
        assert!(TimeRange::new(-5.0, 10.0).is_err());
    }

    #[test]
    fn test_drop_middle_split() {
        let mut tl = Timeline::new(100.0);
        tl.apply_drop(40.0, 60.0);

        assert_eq!(tl.segments.len(), 2);
        assert_eq!(tl.segments[0].start, 0.0);
        assert_eq!(tl.segments[0].end, 40.0);
        assert_eq!(tl.segments[1].start, 60.0);
        assert_eq!(tl.segments[1].end, 100.0);
    }

    #[test]
    fn test_drop_full_coverage() {
        let mut tl = Timeline::new(50.0);
        tl.apply_drop(0.0, 60.0);

        assert_eq!(tl.segments.len(), 0);
    }

    #[test]
    fn test_normalize_merges_close_segments() {
        let mut tl = Timeline {
            total_duration: 100.0,
            segments: vec![
                TimeRange::new(0.0, 10.0).unwrap(),
                TimeRange::new(10.02, 20.0).unwrap(),
            ],
        };
        tl.normalize();

        assert_eq!(tl.segments.len(), 1);
        assert_eq!(tl.segments[0].start, 0.0);
        assert_eq!(tl.segments[0].end, 20.0);
    }
}