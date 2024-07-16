use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};
use crate::app::{App, AppResult};
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {

    let area = utils::centered_rect(60, 40, frame.size());

    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Min(1),
        Constraint::Length(3),
    ]).split(area);

    let popup_block = Paragraph::new(format!(
        "Testing details:\n\
                Salt: {}\n\
                Token: {}\n\
                Server address: {}\n\
                Connection status: {}\n\
                Last connection: {}\n\
                Connection error code: {}\n\
                Connection message: {}\n\
                ",
        app.server.salt,
        app.server.token,
        app.server.server_address,
        app.server.connection_status,
        app.server.last_connection_timestamp,
        app.server.connection_code,
        app.server.connection_message
    )).wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("Test Navidrome server")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::default()).bg(Color::default()));

    let popup_footer = Paragraph::new(Line::from("(r) to generate new salt and token (t) to test connection"))
        .block(Block::default());

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, chunks[0]);
    frame.render_widget(popup_footer, chunks[1]);
    Ok(())
}