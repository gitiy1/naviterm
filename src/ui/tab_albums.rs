use crate::app::{App, AppResult};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Modifier;
use ratatui::style::Color::{Yellow};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem};
use ratatui::Frame;
use crate::ui::utils::get_text_for_album_item;

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

    let mut items: Vec<ListItem> = Vec::new();
    for (index, album_id) in list.iter().enumerate() {
        items.push(get_text_for_album_item(
            &app.database,
            &app.app_flags,
            app.list_states.album_state.selected().unwrap(),
            index,
            album_id,
            &app.search_data,
            app.home_pane.to_u8(),
            app.home_pane.to_u8(),
            true
        ));
    }
    let list = List::new(items)
        .block(results_block)
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);

    if app.app_flags.move_to_next_in_search {
        app.app_flags.move_to_next_in_search = false;
        app.list_states.album_state.select(Some(
            *app.search_data.search_results_indexes.get(app.search_data.index_in_search).unwrap(),
        ));
    }
    frame.render_stateful_widget(list, chunks[1], &mut app.list_states.album_state);

    Ok(())
}
