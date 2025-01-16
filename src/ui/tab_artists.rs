use crate::app::{App, AppResult, TwoPaneVertical};
use crate::ui::utils::{
    get_text_for_album_item, get_text_for_artist_item, get_text_for_song_item, FormatFlags,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Alignment;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let mut block_artists = Block::bordered()
        .border_type(Rounded)
        .title(Line::raw("Artists").left_aligned())
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut block_artist_selected = Block::bordered()
        .border_type(Rounded)
        .title(Line::raw("Albums by selected artist").left_aligned())
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let active_pane_style = Style::default().fg(app.app_colors.primary_accent);

    match app.artist_pane {
        TwoPaneVertical::Left => {
            block_artists = block_artists.border_style(active_pane_style);
        }
        TwoPaneVertical::Right => {
            block_artist_selected = block_artist_selected.border_style(active_pane_style);
        }
    }

    if app.database.artists().is_empty() || app.database.alphabetical_artists().is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No artists..."))
                .style(Style::default().fg(app.app_colors.secondary_accent))
                .alignment(Alignment::Center)
                .block(Block::bordered().border_type(Rounded)),
            area,
        );
    } else {
        let mut artists_items: Vec<ListItem> = Vec::new();
        for (index, artist_id) in app.database.alphabetical_artists().iter().enumerate() {
            artists_items.push(get_text_for_artist_item(
                &app.database,
                &app.app_flags,
                &app.app_colors,
                app.list_states.artist_state.selected().unwrap(),
                index,
                artist_id,
                &app.search_data,
                app.artist_pane.to_u8(),
                TwoPaneVertical::Left as u8,
            ));
        }
        let list = List::new(artists_items)
            .block(block_artists)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        if app.app_flags.move_to_next_in_search && app.artist_pane == TwoPaneVertical::Left {
            app.app_flags.move_to_next_in_search = false;
            app.list_states.artist_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }

        frame.render_stateful_widget(list, chunks[0], &mut app.list_states.artist_state);

        let selected_artist = app.database.get_artist(
            app.database
                .alphabetical_artists()
                .get(app.list_states.artist_state.selected().unwrap())
                .unwrap(),
        );

        let mut album_items: Vec<ListItem> = vec![];
        let mut index = 0;
        for album_id in selected_artist.albums() {
            let album = app.database.get_album(album_id);
            let format_flags = FormatFlags {
                include_artist: false,
                include_track: true,
                indent: true,
                highlight_title: true,
            };
            album_items.push(get_text_for_album_item(
                &app.database,
                &app.app_flags,
                &app.app_colors,
                app.list_states.artist_selected_state.selected().unwrap(),
                index,
                album_id,
                &app.search_data,
                app.artist_pane.to_u8(),
                TwoPaneVertical::Right as u8,
                &format_flags,
            ));
            for song_id in album.songs() {
                index += 1;
                album_items.push(get_text_for_song_item(
                    &app.database,
                    &app.app_flags,
                    &app.app_colors,
                    app.list_states.artist_selected_state.selected(),
                    index,
                    song_id,
                    &app.search_data,
                    app.artist_pane.to_u8(),
                    TwoPaneVertical::Right as u8,
                    &format_flags,
                ));
            }
            index += 1;
        }

        let list = List::new(album_items)
            .block(block_artist_selected)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        if app.app_flags.move_to_next_in_search && app.artist_pane == TwoPaneVertical::Right {
            app.app_flags.move_to_next_in_search = false;
            app.list_states.artist_selected_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }

        frame.render_stateful_widget(list, chunks[1], &mut app.list_states.artist_selected_state);
    }

    Ok(())
}
