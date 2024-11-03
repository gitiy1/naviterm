use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, AppResult};
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.size());

    let popup_block = Paragraph::new(format!(
        "(b) Albums\n\
         (p) Playlists\n\
         (g) Genres\n\
         (a) All\n\
        "
    ))
    .wrap(Wrap { trim: true })
    .block(
        Block::bordered()
            .title("Update database")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(4, 4, 1, 1)),
    )
    .style(Style::default().fg(Color::default()).bg(Color::default()));

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, area);

    Ok(())
}
