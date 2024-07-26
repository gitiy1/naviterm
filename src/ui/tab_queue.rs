use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color::{Gray, Yellow};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult};
use crate::ui::utils::duration_to_hhmmss;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    

    let block = Block::bordered()
        .border_type(Rounded);
    

    if app.queue.is_empty() {
        frame.render_widget(Paragraph::new(
            Line::from("Nothing in the queue...")).block(Block::default()).block(block),area);
    }
    else {
        let items = app.queue.iter().enumerate()
            .map(|(i, song_id)| {
                let song = app.database.get_song(song_id);
                let song_item = Text::from(vec![
                    Line::from(vec![
                        Span { content: song.title().into(), style: Style::default().fg(Yellow) },
                        Span { content: " (".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: duration_to_hhmmss(song.duration()).into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: ")".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ])
                ]);
                if i == app.index_in_queue {
                    ListItem::from(song_item).underlined()
                }
                else {
                    ListItem::from(song_item)
                }
            });
        let list = List::new(items).block(block).highlight_symbol("-> ").highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(list, area, &mut app.queue_list_state);
    }

    Ok(())
}