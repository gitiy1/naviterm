use crate::app::{App, AppHomeTabMode, AppResult, HomePane};
use crate::ui::utils::{get_text_for_album_item, get_text_for_song_item};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::style::{Style};
use ratatui::text::{Line};
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
        .style(Style::default().fg(Gray));

    let mut block_most_listened = Block::bordered()
        .title(Line::raw("Most listened albums").left_aligned())
        .border_type(Rounded)
        .style(Style::default().fg(Gray));

    let mut block_recently_added = Block::bordered()
        .title(Line::raw("Recently added albums").left_aligned())
        .border_type(Rounded)
        .style(Style::default().fg(Gray));

    let mut block_most_listened_tracks = Block::bordered()
        .title(Line::raw("Most listened tracks").left_aligned())
        .border_type(Rounded)
        .style(Style::default().fg(Gray));

    let active_pane_style = Style::default().fg(Yellow);

    match app.home_pane {
        HomePane::Top => {
            block_recents = block_recents.style(active_pane_style);
        }
        HomePane::Bottom => {
            block_most_listened = block_most_listened.style(active_pane_style);
        }
        HomePane::TopLeft => {
            block_recents = block_recents.style(active_pane_style);
        }
        HomePane::TopRight => {
            block_recently_added = block_recently_added.style(active_pane_style);
        }
        HomePane::BottomLeft => {
            block_most_listened = block_most_listened.style(active_pane_style);
        }
        HomePane::BottomRight => {
            block_most_listened_tracks = block_most_listened_tracks.style(active_pane_style);
        }
    }

    if recent_albums.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No recent albums..."))
                .block(Block::default())
                .block(block_recents),
            chunks_top[0],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        for (index, album_id) in recent_albums.iter().enumerate() {
            items.push(get_text_for_album_item(
                &app.database,
                &app.app_flags,
                index,
                album_id,
                &app.search_data,
                &app.home_pane,
                &HomePane::TopLeft,
            ));
        }
        let list = List::new(items)
            .block(block_recents)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        let list_state = match app.home_tab_mode {
            AppHomeTabMode::OneColumn => {
                if app.list_states.home_tab_top.selected().is_none() {
                    app.list_states.home_tab_top.select_first();
                }
                &mut app.list_states.home_tab_top
            }
            AppHomeTabMode::TwoColumns => {
                if app.list_states.home_tab_top_left.selected().is_none() {
                    app.list_states.home_tab_top_left.select_first();
                }
                &mut app.list_states.home_tab_top_left
            }
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
                .block(block_most_listened),
            chunks_bottom[0],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        for (index, album_id) in most_listened_albums.iter().enumerate() {
            items.push(get_text_for_album_item(
                &app.database,
                &app.app_flags,
                index,
                album_id,
                &app.search_data,
                &app.home_pane,
                &HomePane::BottomLeft,
            ));
        }
        let list = List::new(items)
            .block(block_most_listened)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        let list_state = match app.home_tab_mode {
            AppHomeTabMode::OneColumn => {
                if app.list_states.home_tab_bottom.selected().is_none() {
                    app.list_states.home_tab_bottom.select_first();
                }
                &mut app.list_states.home_tab_bottom
            }
            AppHomeTabMode::TwoColumns => {
                if app.list_states.home_tab_bottom_left.selected().is_none() {
                    app.list_states.home_tab_bottom_left.select_first();
                }
                &mut app.list_states.home_tab_bottom_left
            }
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
                .block(block_most_listened_tracks),
            chunks_bottom[1],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        for (index, song_id) in most_listened_tracks.iter().enumerate() {
            items.push(get_text_for_song_item(
                &app.database,
                &app.app_flags,
                index,
                song_id,
                &app.search_data,
                app.home_pane.to_u8(),
                HomePane::BottomRight as u8,
            ));
        }
        let list = List::new(items)
            .block(block_most_listened_tracks)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        if app.list_states.home_tab_bottom_right.selected().is_none() {
            app.list_states.home_tab_bottom_right.select_first()
        } else if app.app_flags.move_to_next_in_search && app.home_pane == HomePane::BottomRight {
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
                .block(block_recently_added),
            chunks_top[1],
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        for (index, album_id) in recently_added_albums.iter().enumerate() {
            items.push(get_text_for_album_item(
                &app.database,
                &app.app_flags,
                index,
                album_id,
                &app.search_data,
                &app.home_pane,
                &HomePane::TopRight,
            ));
        }
        let list = List::new(items)
            .block(block_recently_added)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        let list_state = match app.home_tab_mode {
            AppHomeTabMode::OneColumn => {
                // TODO: handle this appropriately
                if app.list_states.home_tab_bottom.selected().is_none() {
                    app.list_states.home_tab_bottom.select_first();
                }
                &mut app.list_states.home_tab_bottom
            }
            AppHomeTabMode::TwoColumns => {
                if app.list_states.home_tab_top_right.selected().is_none() {
                    app.list_states.home_tab_top_right.select_first();
                }
                &mut app.list_states.home_tab_top_right
            }
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
