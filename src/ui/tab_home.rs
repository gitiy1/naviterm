use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult, HomePane};

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {

    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).split(area);

    let recent_albums = app.database.recent_albums();
    let most_listened_albums = app.database.most_listened_albums();

    let mut block_recents = Block::bordered()
        .title(Line::raw("Recent albums").left_aligned())
        .border_type(Rounded).style(Style::default().fg(Gray));

    let mut block_most_listened = Block::bordered()
        .title(Line::raw("Most listened albums").left_aligned())
        .border_type(Rounded).style(Style::default().fg(Gray));

    let active_pane_style = Style::default().fg(Yellow);

    match app.home_pane {
        HomePane::Top => { block_recents = block_recents.style(active_pane_style); }
        HomePane::Bottom => { block_most_listened = block_most_listened.style(active_pane_style); }
    }

    if recent_albums.is_empty() {
        frame.render_widget(Paragraph::new(
            Line::from("No recent albums...")).block(Block::default()).block(block_recents),area);
    }
    else {
        let items = recent_albums.iter().enumerate()
            .map(|(_i, album_id)| {
                let album = app.database.get_album(album_id);
                let album_item = Text::from(vec![
                    Line::from(vec![
                        Span { content: album.name().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
                        Span { content: " from ".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: album.artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ]),
                    Line::from(vec![
                        Span { content: album.genres().join(", ").into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: ", ".into(), style: Style::default() },
                        Span { content: album.song_count().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: " songs".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ])
                ]);
                ListItem::from(album_item)
            });
        let list = List::new(items).block(block_recents).highlight_symbol("-> ").highlight_spacing(HighlightSpacing::Always);
        if app.home_top_state.selected().is_none() { app.home_top_state.select_first() }
        frame.render_stateful_widget(list, chunks[0], &mut app.home_top_state);
    }

    if most_listened_albums.is_empty() {
        frame.render_widget(Paragraph::new(
            Line::from("No most listened albums...")).block(Block::default()).block(block_most_listened),area);
    }
    else {
        let items = most_listened_albums.iter().enumerate()
            .map(|(_i, album_id)| {
                let album = app.database.get_album(album_id);
                let album_item = Text::from(vec![
                    Line::from(vec![
                        Span { content: album.name().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
                        Span { content: " from ".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: album.artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ]),
                    Line::from(vec![
                        Span { content: album.genres().join(", ").into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: ", ".into(), style: Style::default() },
                        Span { content: album.song_count().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: " songs".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ])
                ]);
                ListItem::from(album_item)
            });
        let list = List::new(items).block(block_most_listened).highlight_symbol("-> ").highlight_spacing(HighlightSpacing::Always);
        if app.home_bottom_state.selected().is_none() { app.home_bottom_state.select_first() }
        frame.render_stateful_widget(list, chunks[1], &mut app.home_bottom_state);
    }
    Ok(())
}