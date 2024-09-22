use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Modifier;
use ratatui::style::{Style};
use ratatui::style::Color::{Yellow};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block};
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

    frame.render_widget(results_block, chunks[1]);
    
    Ok(())
}