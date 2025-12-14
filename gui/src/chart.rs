use iced::{
    widget::canvas, mouse, Color, Point, Rectangle, Theme
};
use crate::style::{C_MAYA, C_SKY, C_TEXT}; // Removed C_BG

pub struct AnimatedChart {
    pub data: Vec<(String, i64)>,
    pub progress: f32,
}

impl canvas::Program<()> for AnimatedChart {
    type State = ();

    fn draw(&self, _state: &(), renderer: &iced::Renderer, _theme: &Theme, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        if self.data.is_empty() { return vec![frame.into_geometry()]; }

        let max_val = self.data.iter().map(|(_, v)| *v).max().unwrap_or(3600).max(60) as f32;
        let pad_x = 20.0;
        let pad_bot = 30.0;
        let gap = 15.0;
        let w = bounds.width - (pad_x * 2.0);
        let count = self.data.len() as f32;
        let bar_w = (w - (gap * (count - 1.0))) / count;
        let h_draw = bounds.height - pad_bot;

        let cursor_pos = cursor.position_in(bounds);

        for (i, (label, val)) in self.data.iter().enumerate() {
            let target_h = (*val as f32 / max_val) * h_draw;
            let current_h = target_h * self.progress;

            let x = pad_x + (i as f32 * (bar_w + gap));
            let y = bounds.height - pad_bot - current_h;

            let bar_rect = Rectangle { x, y, width: bar_w, height: current_h };
            let hit_rect = Rectangle { y: bounds.height - pad_bot - target_h, height: target_h, ..bar_rect };
            let is_hovered = cursor_pos.map(|p| hit_rect.contains(p)).unwrap_or(false);

            let color = if is_hovered { C_SKY } else { C_MAYA };
            
            // Rounded Top
            let r = 6.0;
            let path = canvas::Path::new(|p| {
                p.move_to(Point::new(x, y + current_h));
                p.line_to(Point::new(x, y + r));
                p.quadratic_curve_to(Point::new(x, y), Point::new(x + r, y));
                p.line_to(Point::new(x + bar_w - r, y));
                p.quadratic_curve_to(Point::new(x + bar_w, y), Point::new(x + bar_w, y + r));
                p.line_to(Point::new(x + bar_w, y + current_h));
                p.close();
            });
            frame.fill(&path, color);

            // Labels
            let mut txt = canvas::Text::from(label.clone());
            txt.color = C_TEXT;
            txt.size = 12.0.into();
            txt.horizontal_alignment = iced::alignment::Horizontal::Center;
            txt.position = Point::new(x + bar_w/2.0, bounds.height - 20.0);
            frame.fill_text(txt);
            
            if is_hovered && self.progress > 0.8 {
                let dur = format_dur(*val);
                let mut tool = canvas::Text::from(dur);
                tool.color = Color::WHITE;
                tool.size = 14.0.into();
                tool.position = Point::new(x + bar_w/2.0, y - 20.0);
                tool.horizontal_alignment = iced::alignment::Horizontal::Center;
                frame.fill_text(tool);
            }
        }
        vec![frame.into_geometry()]
    }
}

fn format_dur(s: i64) -> String {
    let h = s/3600; let m = (s%3600)/60; 
    if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}