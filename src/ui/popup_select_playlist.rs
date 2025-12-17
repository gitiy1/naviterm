use crate::app::{App, AppResult};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Modifier, Span};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(50, 40, frame.area());

    let new_playlist_style = if app
        .list_states
        .popup_select_playlist_list_state
        .selected()
        .is_some_and(|x| x == 0)
    {
        Style::default()
            .fg(app.app_colors.primary_accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let new_playlist_entry = if app.app_flags.is_introducing_new_playlist_name {
        Line::from(format!("New Playlist (type new name) => {} ", app.new_name))
            .style(new_playlist_style)
    } else {
        Line::from("New Playlist".to_string()).style(new_playlist_style)
    };

    let mut items = vec![ListItem::from(Text::from(new_playlist_entry))];
    for (i, playlist_id) in app.database.alphabetical_playlists().iter().enumerate() {
        let playlist = app.database.get_playlist(playlist_id);
        let song_count_span = Span {
            content: (" - ".to_owned() + playlist.song_count() + " songs").into(),
            style: Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        };
        let item_style = if app
            .list_states
            .popup_select_playlist_list_state
            .selected()
            .unwrap()
            == i + 1
        {
            Style::default()
                .fg(app.app_colors.primary_accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::from(Text::from(vec![Line::from(vec![
            Span {
                content: playlist.name().into(),
                style: item_style,
            },
            song_count_span,
        ])])));
    }

    if app
        .list_states
        .popup_select_playlist_list_state
        .selected()
        .is_none()
    {
        app.list_states
            .popup_select_playlist_list_state
            .select_first()
    }
    let popup_list = List::new(items)
        .style(Style::default().fg(Color::default()))
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);
    let popup_footer = Paragraph::new(
        Line::from(format!(
            "{} select playlist",
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupPlaylistAcceptSelected, None)
        ))
        .style(
            Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        ),
    )
    .centered();

    let block = Block::bordered()
        .title(Line::raw("Add to playlist").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(Clear, area);
    frame.render_stateful_widget(
        popup_list,
        chunks[0],
        &mut app.list_states.popup_select_playlist_list_state,
    );
    frame.render_widget(popup_footer, chunks[1]);
    frame.render_widget(block, area);
    Ok(())
}
