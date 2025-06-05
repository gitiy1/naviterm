use crate::app::{App, AppHomeTabMode, AppResult, HomePane};
use crate::ui::utils::{get_text_for_album_item, get_text_for_song_item, FormatFlags};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let chunks_top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);
    let chunks_bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let recent_albums = app.database.recent_albums();
    let recently_added_albums = app.database.recently_added_albums();
    let most_listened_albums = app.database.most_listened_albums();
    let most_listened_tracks = app.database.most_listened_tracks();

    let mut block_recents = Block::bordered()
        .title(Line::raw("Recently listened albums").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut block_most_listened = Block::bordered()
        .title(Line::raw("Most listened albums").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut block_recently_added = Block::bordered()
        .title(Line::raw("Recently added albums").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut block_most_listened_tracks = Block::bordered()
        .title(Line::raw("Most listened tracks").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let active_pane_style = Style::default().fg(app.app_colors.primary_accent);

    match app.home_pane {
        HomePane::Top => {
            block_recents = block_recents.border_style(active_pane_style);
        }
        HomePane::Bottom => {
            block_most_listened = block_most_listened.border_style(active_pane_style);
        }
        HomePane::TopLeft => {
            block_recents = block_recents.border_style(active_pane_style);
        }
        HomePane::TopRight => {
            block_recently_added = block_recently_added.border_style(active_pane_style);
        }
        HomePane::BottomLeft => {
            block_most_listened = block_most_listened.border_style(active_pane_style);
        }
        HomePane::BottomRight => {
            block_most_listened_tracks = block_most_listened_tracks.border_style(active_pane_style);
        }
    }

    if recent_albums.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No recent albums..."))
                .block(Block::default())
                .style(Style::default().fg(app.app_colors.secondary_accent))
                .alignment(Alignment::Center)
                .block(block_recents),
            chunks_top[0],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        let format_flags = FormatFlags {
            include_artist: true,
            include_track: false,
            indent: false,
            highlight_title: false,
        };
        for (index, album_id) in recent_albums.iter().take(app.app_config.list_size).enumerate() {
            items.push(get_text_for_album_item(
                &app.database,
                &mut app.app_flags,
                &app.app_colors,
                app.list_states.home_tab_top_left.selected().unwrap(),
                index,
                album_id,
                &app.search_data,
                app.home_pane.to_u8(),
                HomePane::TopLeft as u8,
                &format_flags,
            ));
        }
        let list = List::new(items)
            .block(block_recents)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        let list_state = match app.home_tab_mode {
            AppHomeTabMode::OneColumn => &mut app.list_states.home_tab_top,
            AppHomeTabMode::TwoColumns => &mut app.list_states.home_tab_top_left,
        };
        if app.app_flags.move_to_next_in_search && app.home_pane == HomePane::TopLeft {
            app.app_flags.move_to_next_in_search = false;
            list_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }
        frame.render_stateful_widget(list, chunks_top[0], list_state);
    }

    if most_listened_albums.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No most listened albums..."))
                .block(Block::default())
                .style(Style::default().fg(app.app_colors.secondary_accent))
                .alignment(Alignment::Center)
                .block(block_most_listened),
            chunks_bottom[0],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        let format_flags = FormatFlags {
            include_artist: true,
            include_track: false,
            indent: false,
            highlight_title: false,
        };
        for (index, album_id) in most_listened_albums.iter().take(app.app_config.list_size).enumerate() {
            items.push(get_text_for_album_item(
                &app.database,
                &mut app.app_flags,
                &app.app_colors,
                app.list_states.home_tab_bottom_left.selected().unwrap(),
                index,
                album_id,
                &app.search_data,
                app.home_pane.to_u8(),
                HomePane::BottomLeft as u8,
                &format_flags,
            ));
        }
        let list = List::new(items)
            .block(block_most_listened)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        let list_state = match app.home_tab_mode {
            AppHomeTabMode::OneColumn => &mut app.list_states.home_tab_bottom,
            AppHomeTabMode::TwoColumns => &mut app.list_states.home_tab_bottom_left,
        };
        if app.app_flags.move_to_next_in_search && app.home_pane == HomePane::BottomLeft {
            app.app_flags.move_to_next_in_search = false;
            list_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }
        frame.render_stateful_widget(list, chunks_bottom[0], list_state);
    }

    if most_listened_tracks.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No most listened tracks..."))
                .block(Block::default())
                .style(Style::default().fg(app.app_colors.secondary_accent))
                .alignment(Alignment::Center)
                .block(block_most_listened_tracks),
            chunks_bottom[1],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        let format_flags = FormatFlags {
            include_artist: true,
            include_track: false,
            indent: false,
            highlight_title: false,
        };
        for (index, song_id) in most_listened_tracks.iter().take(app.app_config.list_size).enumerate() {
            items.push(get_text_for_song_item(
                &app.database,
                &mut app.app_flags,
                &app.app_colors,
                app.list_states.home_tab_bottom_right.selected(),
                index,
                song_id,
                &app.search_data,
                app.home_pane.to_u8(),
                HomePane::BottomRight as u8,
                &format_flags,
            ));
        }
        let list = List::new(items)
            .block(block_most_listened_tracks)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        if app.app_flags.move_to_next_in_search && app.home_pane == HomePane::BottomRight {
            app.app_flags.move_to_next_in_search = false;
            app.list_states.home_tab_bottom_right.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }
        frame.render_stateful_widget(
            list,
            chunks_bottom[1],
            &mut app.list_states.home_tab_bottom_right,
        );
    }

    if recently_added_albums.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No recently added albums..."))
                .block(Block::default())
                .style(Style::default().fg(app.app_colors.secondary_accent))
                .alignment(Alignment::Center)
                .block(block_recently_added),
            chunks_top[1],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        let format_flags = FormatFlags {
            include_artist: true,
            include_track: false,
            indent: false,
            highlight_title: false,
        };
        for (index, album_id) in recently_added_albums.iter().take(app.app_config.list_size).enumerate() {
            items.push(get_text_for_album_item(
                &app.database,
                &mut app.app_flags,
                &app.app_colors,
                app.list_states.home_tab_top_right.selected().unwrap(),
                index,
                album_id,
                &app.search_data,
                app.home_pane.to_u8(),
                HomePane::TopRight as u8,
                &format_flags,
            ));
        }
        let list = List::new(items)
            .block(block_recently_added)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        let list_state = match app.home_tab_mode {
            AppHomeTabMode::OneColumn => {
                // TODO: handle this appropriately
                &mut app.list_states.home_tab_bottom
            }
            AppHomeTabMode::TwoColumns => &mut app.list_states.home_tab_top_right,
        };
        if app.app_flags.move_to_next_in_search && app.home_pane == HomePane::TopRight {
            app.app_flags.move_to_next_in_search = false;
            list_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }
        frame.render_stateful_widget(list, chunks_top[1], list_state);
    }

    Ok(())
}
