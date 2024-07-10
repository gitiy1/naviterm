use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use crate::app::{App, AppResult};

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    
    let recent_albums = app.database.recent_albums();
    
    if recent_albums.is_empty() {
        frame.render_widget(Paragraph::new(
            Line::from("No recent albums...")).block(Block::default()),area);
    }

    Ok(())
}