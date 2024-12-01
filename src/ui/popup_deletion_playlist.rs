use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, AppResult};
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.size());

    let selected_playlist = app.database.get_playlist(
        app.database
            .alphabetical_playlists()
            .get(app.list_states.playlist_state.selected().unwrap())
            .unwrap(),
    );

    let popup_content = if selected_playlist.id().starts_with("local") {
        Paragraph::new(format!(
            "Are you sure you want to delete playlist {}?\n\n\
            (y) Yes\n\
            (n) No",
            selected_playlist.name()
        ))
    } else {
        Paragraph::new(format!(
            "Are you sure you want to delete playlist {}? (Note: this will also \
            delete the server playlist!)\n\n\
            (y) Yes\n\
            (n) No",
            selected_playlist.name()
        ))
    };

    let popup_block = popup_content
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("Confirm playlist deletion")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .padding(Padding::new(4, 4, 1, 1)),
        )
        .style(Style::default().fg(Color::default()).bg(Color::default()));

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, area);

    Ok(())
}
