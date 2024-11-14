use crate::app::{App, AppResult};
use log::debug;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Modifier;
use ratatui::style::Color::{Black, Gray, Yellow};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List};
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

    let filter_text = Line::from(vec![
        Span {
            content: "Genre: ".into(),
            style: Style::default(),
        },
        Span {
            content: app.album_genre_filter.clone().into(),
            style: Style::default().fg(Yellow),
        },
        Span {
            content: ", year: ".into(),
            style: Style::default(),
        },
        Span {
            content: app.album_year_filter.clone().into(),
            style: Style::default().fg(Yellow),
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
            content: app.album_sorting_mode.clone().into(),
            style: Style::default().fg(Yellow),
        },
        Span {
            content: ", direction: ".into(),
            style: Style::default(),
        },
        Span {
            content: app.album_sorting_direction.clone().into(),
            style: Style::default().fg(Yellow),
        },
    ])
    .style(Style::default().add_modifier(Modifier::ITALIC));

    frame.render_widget(sorting_text, sorting_area);
    frame.render_widget(sorting_block, header_chunks[1]);

    let results_block = Block::bordered()
        .title(Line::raw("Albums").left_aligned())
        .border_type(Rounded)
        .style(Style::default());

    let list = if app.album_genre_filter == "any" {
        if app.album_sorting_mode == "frequent" {
            app.database.most_listened_albums()
        } else {
            app.database.alphabetical_list_albums()
        }
    } else {
        app.database.filtered_albums()
    };

    let mut album_vector = Vec::new();
    for (index, album_id) in list.iter().enumerate() {
        let album = app.database.get_album(album_id);
        let album_name_to_search = if app.upper_case_search {
            album.name().to_string()
        } else {
            album.name().to_lowercase()
        };
        let mut album_first_line_vector: Vec<Span> = vec![];
        let mut album_second_line_vector: Vec<Span> = vec![];
        if !app.search_results_indexes.is_empty()
            && index == *app.search_results_indexes.get(app.index_in_search).unwrap()
        {
            debug!("album: {}, search string: {}, album index: {}, app search index: {}, search matches indexes: {:?}", album_name_to_search, app.search_string, index, app.index_in_search, app.search_results_indexes);
            let match_indices: Vec<_> = album_name_to_search
                .match_indices(app.search_string.as_str())
                .collect();
            debug!("match: {:?}", match_indices);
            let (first_index, first_match) = match_indices[0];
            let first_slice = &album.name()[0..first_index];
            let matched_slice = &album.name()[first_index..first_index + first_match.len()];
            let last_slice = &album.name()[first_index + first_match.len()..];
            album_first_line_vector.push(
                Span::from(first_slice)
                    .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
            );
            album_first_line_vector.push(
                Span::from(matched_slice).style(
                    Style::default()
                        .fg(Black)
                        .bg(Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            );
            album_first_line_vector.push(
                Span::from(last_slice)
                    .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
            );
        } else {
            album_first_line_vector.push(
                Span::from(album.name())
                    .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
            )
        }
        album_first_line_vector.push(Span {
            content: " from ".into(),
            style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
        });
        album_first_line_vector.push(Span {
            content: app.database.get_album(album_id).artist().into(),
            style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
        });
        album_second_line_vector.push(Span {
            content: app.database.get_album(album_id).genres().join(", ").into(),
            style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
        });
        album_second_line_vector.push(Span {
            content: ", ".into(),
            style: Style::default(),
        });
        album_second_line_vector.push(Span {
            content: app.database.get_album(album_id).song_count().into(),
            style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
        });
        album_second_line_vector.push(Span {
            content: " songs".into(),
            style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
        });
        let album_item = Text::from(vec![
            Line::from(album_first_line_vector),
            Line::from(album_second_line_vector),
        ]);
        album_vector.push(album_item);
    }

    let list = List::new(album_vector)
        .block(results_block)
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);

    if app.album_state.selected().is_none() {
        app.album_state.select_first()
    } else if app.move_to_next_in_search {
        app.move_to_next_in_search = false;
        app.album_state.select(Some(
            *app.search_results_indexes.get(app.index_in_search).unwrap(),
        ));
    }
    frame.render_stateful_widget(list, chunks[1], &mut app.album_state);

    Ok(())
}
