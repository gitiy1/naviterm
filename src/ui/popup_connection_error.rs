use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{AppResult};
use crate::ui::utils;

pub fn draw_popup(frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.size());

    let popup_content = 
        Paragraph::new("It seems that there was a connection error while performing an operation \
        with the server (check the logs for more details). You can now retry or switch to offline \
        mode. \n\n\
        (r) Retry\n\
        (o) Offline".to_string());

    let popup_block = popup_content
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("Connection error")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .padding(Padding::new(4, 4, 1, 1)),
        )
        .style(Style::default().fg(Color::default()).bg(Color::default()));

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, area);

    Ok(())
}
