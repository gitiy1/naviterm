use crate::app::{App, AppResult, SortMode, TwoPaneVertical};
use crate::ui::utils::{
    get_text_for_album_info, get_text_for_album_item, get_text_for_song_item, FormatFlags,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Padding, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let filters_block = Block::bordered()
        .title(Line::raw("Filters").left_aligned())
        .border_type(Rounded)
        .style(Style::default());
    let filter_area = filters_block.inner(header_chunks[0]);

    let year_filter_string = if app.album_filters.year_from_filter.is_empty() {
        "any"
    } else if app.album_filters.year_to_filter.is_empty() {
        app.album_filters.year_from_filter.as_str()
    } else {
        &*format!(
            "{} -> {}",
            app.album_filters.year_from_filter, app.album_filters.year_to_filter
        )
    };

    let filter_text = Line::from(vec![
        Span {
            content: "Genre: ".into(),
            style: Style::default(),
        },
        Span {
            content: app.album_filters.genre_filter.clone().into(),
            style: Style::default().fg(app.app_colors.primary_accent),
        },
        Span {
            content: ", year: ".into(),
            style: Style::default(),
        },
        Span {
            content: year_filter_string.into(),
            style: Style::default().fg(app.app_colors.primary_accent),
        },
    ])
    .style(Style::default().add_modifier(Modifier::ITALIC));

    frame.render_widget(filter_text, filter_area);
    frame.render_widget(filters_block, header_chunks[0]);

    let sorting_block = Block::bordered()
        .title(Line::raw("Sorting").left_aligned())
        .border_type(Rounded)
        .style(Style::default());
    let sorting_area = sorting_block.inner(header_chunks[1]);

    let sorting_text = Line::from(vec![
        Span {
            content: "Mode: ".into(),
            style: Style::default(),
        },
        Span {
            content: app.album_sorting_mode.as_str().into(),
            style: Style::default().fg(app.app_colors.primary_accent),
        },
        Span {
            content: ", order: ".into(),
            style: Style::default(),
        },
        Span {
            content: app.album_sorting_direction.as_str().into(),
            style: Style::default().fg(app.app_colors.primary_accent),
        },
    ])
    .style(Style::default().add_modifier(Modifier::ITALIC));

    frame.render_widget(sorting_text, sorting_area);
    frame.render_widget(sorting_block, header_chunks[1]);

    let albums_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    let mut results_block = Block::bordered()
        .title(Line::raw("Albums").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent));

    let mut info_block = Block::bordered()
        .title(Line::raw("Album Information").left_aligned())
        .border_type(Rounded)
        .border_style(Style::default().fg(app.app_colors.secondary_accent))
        .padding(Padding::new(2, 2, 1, 1));

    let active_pane_style = Style::default().fg(app.app_colors.primary_accent);

    match app.album_pane {
        TwoPaneVertical::Left => {
            results_block = results_block.border_style(active_pane_style);
        }
        TwoPaneVertical::Right => {
            info_block = info_block.border_style(active_pane_style);
        }
    }

    let list = if app.album_filters.genre_filter == "any" && app.album_filters.year_from_filter.is_empty() {
        if app.album_sorting_mode == SortMode::Frequent {
            app.database.most_listened_albums()
        } else {
            app.database.alphabetical_list_albums()
        }
    } else {
        app.database.filtered_albums()
    };

    let mut items: Vec<ListItem> = Vec::new();
    let format_flags = FormatFlags {
        include_artist: false,
        include_track: true,
        indent: false,
        highlight_title: false,
    };
    for (index, album_id) in list.iter().enumerate() {
        items.push(get_text_for_album_item(
            &app.database,
            &app.app_flags,
            &app.app_colors,
            app.list_states.album_state.selected().unwrap(),
            index,
            album_id,
            &app.search_data,
            app.album_pane.to_u8(),
            TwoPaneVertical::Left as u8,
            &format_flags,
        ));
    }
    let album_list = List::new(items)
        .block(results_block)
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);

    if app.app_flags.move_to_next_in_search && app.album_pane == TwoPaneVertical::Left {
        app.app_flags.move_to_next_in_search = false;
        app.list_states.album_state.select(Some(
            *app.search_data
                .search_results_indexes
                .get(app.search_data.index_in_search)
                .unwrap(),
        ));
    }
    frame.render_stateful_widget(album_list, albums_area[0], &mut app.list_states.album_state);

    let info_block_inner = info_block.inner(albums_area[1]);

    let chunks_info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Percentage(100)])
        .split(info_block_inner);

    let selected_album = app.database.get_album(
        list.get(app.list_states.album_state.selected().unwrap())
            .unwrap(),
    );

    let album_info = Paragraph::new(get_text_for_album_info(selected_album, &app.app_colors))
        .wrap(Wrap { trim: true });

    frame.render_widget(album_info, chunks_info[0]);

    let mut song_items: Vec<ListItem> = Vec::new();
    for (index, song_id) in selected_album.songs().iter().enumerate() {
        song_items.push(get_text_for_song_item(
            &app.database,
            &app.app_flags,
            &app.app_colors,
            app.list_states.album_selected_state.selected(),
            index,
            song_id,
            &app.search_data,
            app.album_pane.to_u8(),
            TwoPaneVertical::Right as u8,
            &format_flags,
        ));
    }

    let song_list = List::new(song_items)
        .style(Style::default().fg(Color::default()))
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);

    if app.app_flags.move_to_next_in_search && app.album_pane == TwoPaneVertical::Right {
        app.app_flags.move_to_next_in_search = false;
        app.list_states.album_selected_state.select(Some(
            *app.search_data
                .search_results_indexes
                .get(app.search_data.index_in_search)
                .unwrap(),
        ));
    }

    frame.render_stateful_widget(
        song_list,
        chunks_info[1],
        &mut app.list_states.album_selected_state,
    );
    frame.render_widget(info_block, albums_area[1]);

    Ok(())
}
