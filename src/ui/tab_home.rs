use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color::{Gray, Yellow};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult};

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    
    let recent_albums = app.database.recent_albums();

    let block = Block::bordered()
        .title(Line::raw("Recent albums").left_aligned())
        .border_type(Rounded);

    if recent_albums.is_empty() {
        frame.render_widget(Paragraph::new(
            Line::from("No recent albums...")).block(Block::default()).block(block),area);
    }
    else {
        let items = recent_albums.iter().enumerate()
            .map(|(_i, album)| {
                let album_item = Text::from(vec![
                    Line::from(vec![
                        Span { content: album.name().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
                        Span { content: " from ".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: album.artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ]),
                    Line::from(vec![
                        Span { content: album.genre().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: ", ".into(), style: Style::default() },
                        Span { content: album.song_count().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: " songs".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ])
                ]);
                ListItem::from(album_item)
            });
        let list = List::new(items).block(block).highlight_symbol("-> ").highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(list, area, &mut app.home_recent_state);
    }

    Ok(())
}