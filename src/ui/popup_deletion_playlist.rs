use crate::app::{App, AppResult};
use crate::ui::utils;
use ratatui::layout::Alignment;
use ratatui::prelude::{Line, Modifier, Span};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;
use crate::mappings::ShortcutAction;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.area());

    let selected_playlist = app.database.get_playlist(
        app.database
            .alphabetical_playlists()
            .get(app.list_states.playlist_state.selected().unwrap())
            .unwrap(),
    );

    let mut popup_lines = if selected_playlist.id().starts_with("local") {
        vec![Line::from(format!(
            "Are you sure you want to delete playlist {}",
            selected_playlist.name()
        ))]
    } else {
        vec![Line::from(format!(
            "Are you sure you want to delete playlist {}? (Note: this will also delete the server playlist!)",
            selected_playlist.name()
        ))]
    };

    popup_lines.append(&mut vec![
        Line::from(""),
        Line::from(vec![
            Span {
                content: app.shortcuts.get_key_combo_for_operation(ShortcutAction::PopupConfirmDeletionPlaylistYes, None).into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: " Yes".into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: app.shortcuts.get_key_combo_for_operation(ShortcutAction::PopupConfirmDeletionPlaylistNo, None).into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: " No".into(),
                style: Style::default(),
            },
        ]),
    ]);

    let popup_content = Paragraph::new(popup_lines);

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
