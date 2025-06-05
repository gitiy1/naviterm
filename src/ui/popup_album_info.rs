use crate::app::{App, AppResult, CurrentScreen, HomePane};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use crate::ui::utils::{get_text_for_album_info, get_text_for_song_item, FormatFlags};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(60, 60, frame.size());
    let context = "album_information";

    let album = match app.current_screen {
        CurrentScreen::Home => match app.home_pane {
            HomePane::Top => app.database.get_album(
                app.database
                    .recent_albums()
                    .get(app.list_states.home_tab_top.selected().unwrap())
                    .unwrap(),
            ),
            HomePane::Bottom => app.database.get_album(
                app.database
                    .most_listened_albums()
                    .get(app.list_states.home_tab_bottom.selected().unwrap())
                    .unwrap(),
            ),
            HomePane::TopLeft => app.database.get_album(
                app.database
                    .recent_albums()
                    .get(app.list_states.home_tab_top_left.selected().unwrap())
                    .unwrap(),
            ),
            HomePane::TopRight => app.database.get_album(
                app.database
                    .recently_added_albums()
                    .get(app.list_states.home_tab_top_right.selected().unwrap())
                    .unwrap(),
            ),
            HomePane::BottomLeft => app.database.get_album(
                app.database
                    .most_listened_albums()
                    .get(app.list_states.home_tab_bottom_left.selected().unwrap())
                    .unwrap(),
            ),
            HomePane::BottomRight => app.database.get_album(
                app.database
                    .get_song(
                        app.database
                            .most_listened_tracks()
                            .get(app.list_states.home_tab_bottom_right.selected().unwrap())
                            .unwrap(),
                    )
                    .album_id(),
            ),
        },
        CurrentScreen::Albums => app.database.get_album(
            app.database
                .filtered_albums()
                .get(app.list_states.album_state.selected().unwrap())
                .unwrap(),
        ),
        _ => {
            panic!("Should not reach")
        }
    };

    let album_info = get_text_for_album_info(album, &app.app_colors);

    let format_flags = FormatFlags {
        include_artist: false,
        include_track: true,
        indent: false,
        highlight_title: true,
    };

    let mut items: Vec<ListItem> = Vec::new();
    for (index, song_id) in album.songs().iter().enumerate() {
        items.push(get_text_for_song_item(
            &app.database,
            &mut app.app_flags,
            &app.app_colors,
            app.list_states.popup_list_state.selected(),
            index,
            song_id,
            &app.search_data,
            app.home_pane.to_u8(),
            HomePane::BottomRight as u8,
            &format_flags,
        ));
    }

    if app.list_states.popup_list_state.selected().is_none() {
        app.list_states.popup_list_state.select_first()
    }
    let popup_list = List::new(items)
        .style(Style::default().fg(Color::default()))
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);
    let popup_footer = Paragraph::new(Line::from(format!(
        "{} add selected item {} add whole album",
        app.shortcuts
            .get_key_combo_for_operation(ShortcutAction::GoPopupAddSongTo, Some(context)),
        app.shortcuts
            .get_key_combo_for_operation(ShortcutAction::GoPopupAddAlbumTo, Some(context))
    )))
    .style(
        Style::default()
            .fg(app.app_colors.secondary_accent)
            .add_modifier(Modifier::ITALIC),
    )
    .centered();

    let block = Block::bordered()
        .title(Line::raw("Album details").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(inner);

    let chunks_album = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(5)])
        .split(chunks[0]);

    frame.render_widget(Clear, area);
    frame.render_widget(album_info, chunks_album[0]);
    frame.render_stateful_widget(
        popup_list,
        chunks_album[1],
        &mut app.list_states.popup_list_state,
    );
    frame.render_widget(popup_footer, chunks[1]);
    frame.render_widget(block, area);
    Ok(())
}
