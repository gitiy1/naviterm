use crate::app::{App, AppResult, TwoPaneVertical};
use crate::ui::utils::{
    duration_to_hhmmss, get_text_for_playlist_item, get_text_for_song_item, FormatFlags,
};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Modifier, Span};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let mut block_playlists = Block::bordered()
        .title(Line::raw("Playlists").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(Gray));

    let mut block_playlist_selected = Block::bordered()
        .border_type(Rounded)
        .title(Line::raw("Selected playlist content").left_aligned())
        .border_style(Style::default().fg(Gray));

    let active_pane_style = Style::default().fg(Yellow);

    match app.playlist_pane {
        TwoPaneVertical::Left => {
            block_playlists = block_playlists.border_style(active_pane_style);
        }
        TwoPaneVertical::Right => {
            block_playlist_selected = block_playlist_selected.border_style(active_pane_style);
        }
    }

    if app.database.alphabetical_playlists().is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No playlists...").style(Style::default().fg(Gray)))
                .alignment(Alignment::Center)
                .block(Block::bordered().border_type(Rounded)),
            area,
        );
    } else {
        let mut playlist_items: Vec<ListItem> = Vec::new();
        for (index, playlist_id) in app.database.alphabetical_playlists().iter().enumerate() {
            playlist_items.push(get_text_for_playlist_item(
                &app.database,
                &app.app_flags,
                app.list_states.playlist_state.selected().unwrap(),
                index,
                playlist_id,
                &app.search_data,
                app.playlist_pane.to_u8(),
                TwoPaneVertical::Left as u8,
            ))
        }
        let list = List::new(playlist_items)
            .block(block_playlists)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        if app.app_flags.move_to_next_in_search && app.playlist_pane == TwoPaneVertical::Left {
            app.app_flags.move_to_next_in_search = false;
            app.list_states.playlist_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }

        frame.render_stateful_widget(list, chunks[0], &mut app.list_states.playlist_state);

        let selected_playlist_block_inner = block_playlist_selected.inner(chunks[1]);

        let chunks_selected_playlist = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Percentage(100)])
            .split(selected_playlist_block_inner);

        let selected_playlist = app
            .database
            .playlists()
            .get(
                app.database
                    .alphabetical_playlists()
                    .get(app.list_states.playlist_state.selected().unwrap())
                    .unwrap(),
            )
            .unwrap();
        let selected_playlist_songs = selected_playlist.song_list();

        let playlist_info = Paragraph::new(Line::from(vec![
            Span {
                content: selected_playlist.name().into(),
                style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD),
            },
            Span {
                content: " - ".into(),
                style: Style::default().fg(Gray),
            },
            Span {
                content: selected_playlist_songs.len().to_string().into(),
                style: Style::default().fg(Gray),
            },
            Span {
                content: " songs (".into(),
                style: Style::default().fg(Gray),
            },
            Span {
                content: duration_to_hhmmss(selected_playlist.duration()).into(),
                style: Style::default().fg(Gray),
            },
            Span {
                content: ")".into(),
                style: Style::default().fg(Gray),
            },
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(playlist_info, chunks_selected_playlist[0]);

        let mut items: Vec<ListItem> = Vec::new();
        let format_flags = FormatFlags {
            include_artist: true,
            include_track: false,
            indent: false,
            highlight_title: true,
        };
        for (index, song_id) in selected_playlist_songs.iter().enumerate() {
            items.push(get_text_for_song_item(
                &app.database,
                &app.app_flags,
                app.list_states.playlist_selected_state.selected().unwrap(),
                index,
                song_id,
                &app.search_data,
                app.playlist_pane.to_u8(),
                TwoPaneVertical::Right as u8,
                &format_flags,
            ));
        }

        let list = List::new(items)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        if app.app_flags.move_to_next_in_search && app.playlist_pane == TwoPaneVertical::Right {
            app.app_flags.move_to_next_in_search = false;
            app.list_states.playlist_selected_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }

        frame.render_stateful_widget(
            list,
            chunks_selected_playlist[1],
            &mut app.list_states.playlist_selected_state,
        );

        frame.render_widget(block_playlist_selected, chunks[1]);
    }

    Ok(())
}
