use ratatui::layout::{Alignment};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::prelude::{Modifier, Span};
use crate::app::{App, AppResult, MediaType};
use crate::mappings::ShortcutAction;
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.size());

    let added_item_name = match app.item_to_be_added.media_type  {
        MediaType::Song => {
            format!("track {}", app.item_to_be_added.name)
        }
        MediaType::Album => {
            format!("album {}", app.item_to_be_added.name)
        }
        MediaType::Playlist => {
            format!("playlist {}", app.item_to_be_added.name)
        }
        MediaType::Artist => {
            format!("albums from {}", app.item_to_be_added.name)
        }
    };

    let popup_lines = vec![
        Line::from(format!("Adding {} to...", added_item_name)),
        Line::from(""),
        Line::from(vec![
            Span{
                content: app.shortcuts.get_key_combo_for_operation(ShortcutAction::AddItemNext, None).into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Queue, after current song".into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span{
                content: app.shortcuts.get_key_combo_for_operation(ShortcutAction::AddItemEnd, None).into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Queue, at the end".into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span{
                content: app.shortcuts.get_key_combo_for_operation(ShortcutAction::AddItemPlaylist, None).into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Playlists...".into(),
                style: Style::default(),
            },
        ]),
    ];
    let popup_block = Paragraph::new(popup_lines)
    .wrap(Wrap { trim: true })
    .block(
        Block::bordered()
            .title("Add item to")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(4, 4, 1, 1)),
    )
    .style(Style::default().fg(Color::default()).bg(Color::default()));

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, area);

    Ok(())
}
