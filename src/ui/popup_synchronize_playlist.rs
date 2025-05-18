use crate::app::{App, AppResult};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use ratatui::layout::Alignment;
use ratatui::prelude::{Line, Modifier, Span};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.size());

    let selected_playlist = app.database.get_playlist(
        app.database
            .alphabetical_playlists()
            .get(app.list_states.playlist_state.selected().unwrap())
            .unwrap(),
    );

    let popup_lines = if selected_playlist.id().starts_with("local") {
        vec![
            Line::from(format!(
                "Playlist {} is a local playlist. Do you want to push it to the server?",
                selected_playlist.name()
            )),
            Line::from(""),
            Line::from(vec![
                Span {
                    content: app
                        .shortcuts
                        .get_key_combo_for_operation(
                            ShortcutAction::PopupSynchronizeLocalPlaylistPushYes,
                            None,
                        )
                        .into(),
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
                    content: app
                        .shortcuts
                        .get_key_combo_for_operation(
                            ShortcutAction::PopupSynchronizeLocalPlaylistPushNo,
                            None,
                        )
                        .into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: " No".into(),
                    style: Style::default(),
                },
            ]),
        ]
    } else {
        vec![
        Line::from(format!("Playlist {} is also in the server, do you want to push the local server or pull the server one", selected_playlist.name())),
        Line::from(""),
        Line::from(vec![
            Span{
                content: app.shortcuts.get_key_combo_for_operation(ShortcutAction::PopupSynchronizePlaylistPushLocal, None).into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Push local".into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span{
                content: app.shortcuts.get_key_combo_for_operation(ShortcutAction::PopupSynchronizePlaylistPullRemote, None).into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Pull remote".into(),
                style: Style::default(),
            },
        ])]
    };

    let popup_content = Paragraph::new(popup_lines);

    let popup_block = popup_content
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("Synchronize playlist")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .padding(Padding::new(4, 4, 1, 1)),
        )
        .style(Style::default().fg(Color::default()).bg(Color::default()));

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, area);

    Ok(())
}
