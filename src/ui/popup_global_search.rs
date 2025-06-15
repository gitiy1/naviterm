use crate::app::{App, AppResult, FourPaneGrid};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use crate::ui::utils::{get_text_for_global_search_item};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Constraint::Percentage;
use ratatui::prelude::{Color, Modifier, Span, Style};
use ratatui::text::Line;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{
    Block, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph,
};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(80, 60, frame.size());
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .title(Line::raw("Global search").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(inner);

    let results_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Min(5)])
        .split(chunks[1]);

    let results_rects_top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Percentage(50), Percentage(50)])
        .split(results_rects[0]);

    let results_rects_bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Percentage(50), Percentage(50)])
        .split(results_rects[1]);

    let mut block_songs = Block::bordered()
        .title(Line::raw("Songs").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut block_albums = Block::bordered()
        .title(Line::raw("Albums").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut block_artists = Block::bordered()
        .title(Line::raw("Artists").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut block_playlists = Block::bordered()
        .title(Line::raw("Playlists").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let active_pane_style = Style::default().fg(app.app_colors.primary_accent);

    match app.global_search_pane {
        FourPaneGrid::TopLeft => {
            block_songs = block_songs.border_style(active_pane_style);
        }
        FourPaneGrid::TopRight => {
            block_albums = block_albums.border_style(active_pane_style);
        }
        FourPaneGrid::BottomRight => {
            block_artists = block_artists.border_style(active_pane_style);
        }
        FourPaneGrid::BottomLeft => {
            block_playlists = block_playlists.border_style(active_pane_style);
        }
    }

    let mut search_query: Vec<Span> = vec![];

    if app.app_flags.is_introducing_global_search {
        search_query.push(Span {
            content: "Search for: ".into(),
            style: Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        });
        search_query.push(Span {
            content: app.search_data.global_search_string.as_str().into(),
            style: Style::default()
                .fg(app.app_colors.primary_accent)
                .add_modifier(Modifier::BOLD),
        })
    } else {
        search_query.push(Span {
            content: "Search results for \"".into(),
            style: Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        });
        search_query.push(Span {
            content: app.search_data.global_search_string.as_str().into(),
            style: Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        });
        search_query.push(Span {
            content: "\"".into(),
            style: Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        });
    };

    let popup_header = Paragraph::new(Line::from(search_query));
    frame.render_widget(popup_header, chunks[0]);

    let popup_footer_line = if app.app_flags.is_introducing_global_search {
        Line::from(format!(
            "{} accept current query",
            app.shortcuts.get_key_combo_for_operation(
                ShortcutAction::PopupGlobalSearchAcceptSearchString,
                None
            ),
        ))
        .style(
            Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        )
    } else {
        Line::from(format!(
            "{} play item ,{} add selected item to, {} go to item in according pane",
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupGlobalSearchPlayItem, None),
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupGlobalSearchAddItemTo, None),
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupGlobalSearchGoToAccordingPane, None),
        ))
        .style(
            Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        )
    };

    let popup_footer = Paragraph::new(popup_footer_line).centered();

    frame.render_widget(popup_footer, chunks[2]);
    frame.render_widget(block, area);

    if app.search_data.global_search_string.len() < 3 {
        let text = Paragraph::new("Enter a query to start searching")
            .style(
                Style::default()
                    .fg(app.app_colors.secondary_accent)
                    .add_modifier(Modifier::ITALIC),
            )
            .centered();
        frame.render_widget(text, results_rects[1]);
        return Ok(());
    }

    let mut song_results: Vec<ListItem> = vec![];
    if app.search_data.global_search_song_results.is_empty() {
        frame.render_widget(
            Paragraph::new(
                Line::from("No results...").style(
                    Style::default()
                        .fg(app.app_colors.secondary_accent)
                        .add_modifier(Modifier::ITALIC),
                ),
            )
            .centered()
            .block(block_songs),
            results_rects_top[0],
        );
    } else {
        for (index, id) in app
            .search_data
            .global_search_song_results
            .iter()
            .take(app.app_config.list_size)
            .enumerate()
        {
            song_results.push(get_text_for_global_search_item(
                index,
                id,
                &app.database,
                &app.app_colors,
                app.list_states.global_search_songs.selected().unwrap(),
                &app.search_data,
                &FourPaneGrid::TopLeft,
                (results_rects_top[0].width as usize) - 2,
            ));
        }

        let song_results_list = List::new(song_results)
            .block(block_songs)
            .style(Style::default().fg(Color::default()))
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(
            song_results_list,
            results_rects_top[0],
            &mut app.list_states.global_search_songs,
        );
    };

    let mut album_results: Vec<ListItem> = vec![];
    if app.search_data.global_search_albums_results.is_empty() {
        frame.render_widget(
            Paragraph::new(
                Line::from("No results...").style(
                    Style::default()
                        .fg(app.app_colors.secondary_accent)
                        .add_modifier(Modifier::ITALIC),
                ),
            )
                .centered()
                .block(block_albums),
            results_rects_top[1],
        );
    } else {
        for (index, id) in app
            .search_data
            .global_search_albums_results
            .iter()
            .take(app.app_config.list_size)
            .enumerate()
        {
            album_results.push(get_text_for_global_search_item(
                index,
                id,
                &app.database,
                &app.app_colors,
                app.list_states.global_search_albums.selected().unwrap(),
                &app.search_data,
                &FourPaneGrid::TopRight,
                (results_rects_top[1].width as usize) - 2,
            ));
        }

        let album_results_list = List::new(album_results)
            .block(block_albums)
            .style(Style::default().fg(Color::default()))
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(
            album_results_list,
            results_rects_top[1],
            &mut app.list_states.global_search_albums,
        );
    };

    let mut playlist_results: Vec<ListItem> = vec![];
    if app.search_data.global_search_playlists_results.is_empty() {
        frame.render_widget(
            Paragraph::new(
                Line::from("No results...").style(
                    Style::default()
                        .fg(app.app_colors.secondary_accent)
                        .add_modifier(Modifier::ITALIC),
                ),
            )
                .centered()
                .block(block_playlists),
            results_rects_bottom[0],
        );
    } else {
        for (index, id) in app
            .search_data
            .global_search_playlists_results
            .iter()
            .take(app.app_config.list_size)
            .enumerate()
        {
            playlist_results.push(get_text_for_global_search_item(
                index,
                id,
                &app.database,
                &app.app_colors,
                app.list_states.global_search_playlists.selected().unwrap(),
                &app.search_data,
                &FourPaneGrid::BottomLeft,
                (results_rects_bottom[0].width as usize) - 2,
            ));
        }

        let playlist_results_list = List::new(playlist_results)
            .block(block_playlists)
            .style(Style::default().fg(Color::default()))
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(
            playlist_results_list,
            results_rects_bottom[0],
            &mut app.list_states.global_search_playlists,
        );
    };

    let mut artist_results: Vec<ListItem> = vec![];
    if app.search_data.global_search_artists_results.is_empty() {
        frame.render_widget(
            Paragraph::new(
                Line::from("No results...").style(
                    Style::default()
                        .fg(app.app_colors.secondary_accent)
                        .add_modifier(Modifier::ITALIC),
                ),
            )
                .centered()
                .block(block_artists),
            results_rects_bottom[1],
        );
    } else {
        for (index, id) in app
            .search_data
            .global_search_artists_results
            .iter()
            .take(app.app_config.list_size)
            .enumerate()
        {
            artist_results.push(get_text_for_global_search_item(
                index,
                id,
                &app.database,
                &app.app_colors,
                app.list_states.global_search_artists.selected().unwrap(),
                &app.search_data,
                &FourPaneGrid::BottomRight,
                (results_rects_bottom[1].width as usize) - 2,
            ));
        }

        let artist_results_list = List::new(artist_results)
            .block(block_artists)
            .style(Style::default().fg(Color::default()))
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(
            artist_results_list,
            results_rects_bottom[1],
            &mut app.list_states.global_search_artists
        );
    };

    Ok(())
}
