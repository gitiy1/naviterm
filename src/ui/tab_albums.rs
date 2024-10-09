use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Modifier;
use ratatui::style::{Style};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, HighlightSpacing, List};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult};

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {

    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(3),
        Constraint::Min(5),
    ]).split(area);

    let header_chunks = Layout::default().direction(Direction::Horizontal).constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).split(chunks[0]);
    
    let filters_block = Block::bordered()
        .title(Line::raw("Filters").left_aligned())
        .border_type(Rounded).style(Style::default());
    let filter_area = filters_block.inner(header_chunks[0]);
    
    let filter_text = Line::from(vec![
        Span { content: "Genre: ".into(), style: Style::default()},
        Span { content: app.album_genre_filter.clone().into(), style: Style::default().fg(Yellow)},
        Span { content: ", year: ".into(), style: Style::default()},
        Span { content: app.album_year_filter.clone().into(), style: Style::default().fg(Yellow)},
    ]).style(Style::default().add_modifier(Modifier::ITALIC));
    
    frame.render_widget(filter_text, filter_area);
    frame.render_widget(filters_block, header_chunks[0]);

    let sorting_block = Block::bordered()
        .title(Line::raw("Sorting").left_aligned())
        .border_type(Rounded).style(Style::default());
    let sorting_area = sorting_block.inner(header_chunks[1]);

    let sorting_text = Line::from(vec![
        Span { content: "Mode: ".into(), style: Style::default()},
        Span { content: app.album_sorting_mode.clone().into(), style: Style::default().fg(Yellow)},
        Span { content: ", direction: ".into(), style: Style::default()},
        Span { content: app.album_sorting_direction.clone().into(), style: Style::default().fg(Yellow)},
    ]).style(Style::default().add_modifier(Modifier::ITALIC));

    frame.render_widget(sorting_text, sorting_area);
    frame.render_widget(sorting_block, header_chunks[1]);
    
    let results_block = Block::bordered()
        .title(Line::raw("Albums").left_aligned())
        .border_type(Rounded).style(Style::default());

    let mut album_vector = Vec::new();
        for album_id in app.database.filtered_albums() {
            let album_item = Text::from(vec![
                Line::from(vec![
                    Span { content: app.database.get_album(album_id).name().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
                    Span { content: " from ".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    Span { content: app.database.get_album(album_id).artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                ]),
                Line::from(vec![
                    Span { content: app.database.get_album(album_id).genres().join(", ").into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    Span { content: ", ".into(), style: Style::default() },
                    Span { content: app.database.get_album(album_id).song_count().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    Span { content: " songs".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                ])
            ]);
            album_vector.push(album_item);
    }
    
    let list = List::new(album_vector).block(results_block).highlight_symbol("-> ").highlight_spacing(HighlightSpacing::Always);

    if app.album_state.selected().is_none() { app.album_state.select_first() }
    frame.render_stateful_widget(list, chunks[1], &mut app.album_state);
    
    Ok(())
}