use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Pane layout manager for split views
#[derive(Debug, Clone)]
pub struct PaneLayout {
    splits: Vec<Split>,
}

#[derive(Debug, Clone)]
pub enum Split {
    Horizontal { ratio: u16 },
    Vertical { ratio: u16 },
}

impl PaneLayout {
    pub fn new() -> Self {
        Self { splits: Vec::new() }
    }

    /// Add a horizontal split
    pub fn split_horizontal(&mut self, ratio: u16) {
        self.splits.push(Split::Horizontal { ratio });
    }

    /// Add a vertical split
    pub fn split_vertical(&mut self, ratio: u16) {
        self.splits.push(Split::Vertical { ratio });
    }

    /// Calculate layout rectangles for all panes
    pub fn calculate(&self, area: Rect) -> Vec<Rect> {
        if self.splits.is_empty() {
            return vec![area];
        }

        let mut rects = vec![area];

        for split in &self.splits {
            match split {
                Split::Horizontal { ratio } => {
                    // Split the last rect horizontally
                    if let Some(last) = rects.pop() {
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Percentage(*ratio),
                                Constraint::Percentage(100 - ratio),
                            ])
                            .split(last);
                        // Convert Rc<[Rect]> to Vec<Rect>
                        rects.extend_from_slice(&chunks);
                    }
                }
                Split::Vertical { ratio } => {
                    // Split the last rect vertically
                    if let Some(last) = rects.pop() {
                        let chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([
                                Constraint::Percentage(*ratio),
                                Constraint::Percentage(100 - ratio),
                            ])
                            .split(last);
                        // Convert Rc<[Rect]> to Vec<Rect>
                        rects.extend_from_slice(&chunks);
                    }
                }
            }
        }

        rects
    }
}

impl Default for PaneLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_pane() {
        let layout = PaneLayout::new();
        let area = Rect::new(0, 0, 100, 100);
        let rects = layout.calculate(area);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0], area);
    }

    #[test]
    fn test_horizontal_split() {
        let mut layout = PaneLayout::new();
        layout.split_horizontal(50);
        let area = Rect::new(0, 0, 100, 100);
        let rects = layout.calculate(area);
        assert_eq!(rects.len(), 2);
    }

    #[test]
    fn test_vertical_split() {
        let mut layout = PaneLayout::new();
        layout.split_vertical(50);
        let area = Rect::new(0, 0, 100, 100);
        let rects = layout.calculate(area);
        assert_eq!(rects.len(), 2);
    }
}
