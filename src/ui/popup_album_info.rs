use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, List, ListItem, Padding, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult};
use crate::ui::utils;
use crate::ui::utils::duration_to_hhmmss;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {

    let area = utils::centered_rect(60, 60, frame.size());

    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Min(5),
        Constraint::Length(1),
    ]).split(area);


    let album = app.get_current_album().unwrap();

    let album_info = Text::from(vec![
        Line::from(vec![
            Span { content: album.name().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
        ]),
        Line::from(vec![
            Span { content: album.artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
        ]),
        Line::from(vec![
            Span { content: duration_to_hhmmss(album.duration()).into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
            Span { content: " - ".into(), style: Style::default() },
            Span { content: album.genres().join(", ").into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
            Span { content: " - ".into(), style: Style::default() },
            Span { content: album.song_count().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
            Span { content: " songs".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
        ])
    ]);

    let items = album.songs().iter().enumerate()
        .map(|(_i, song)| {
            let song_item = if song.track().is_empty() {
                Text::from(
                    Line::from(vec![
                        Span { content: song.title().into(), style: Style::default().fg(Yellow) },
                        Span { content: " (".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: duration_to_hhmmss(song.duration()).into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: ")".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ])
                )
            }
            else {
                Text::from(
                    Line::from(vec![
                        Span { content: format!("{:>3}",song.track()).into(), style: Style::default().fg(Gray) },
                        Span { content: ". ".into(), style: Style::default().fg(Gray) },
                        Span { content: song.title().into(), style: Style::default().fg(Yellow) },
                        Span { content: " (".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: duration_to_hhmmss(song.duration()).into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                        Span { content: ")".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                    ])
                )
            };
            ListItem::from(song_item)
        });

    let popup_list = List::new(items).style(Style::default().fg(Color::default()));
    let popup_footer = Paragraph::new(Line::from("(a) add selected item (A) add whole album")).block(Block::default());

    let block = Block::bordered()
        .title(Line::raw("Album details").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let inner = block.inner(chunks[0]);

    let chunks_album = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(4),
        Constraint::Min(5),
    ]).split(inner);

    frame.render_widget(Clear, area);
    frame.render_widget(block, chunks[0]);
    frame.render_widget(album_info, chunks_album[0]);
    frame.render_widget(popup_list, chunks_album[1]);
    frame.render_widget(popup_footer, chunks[1]);
    Ok(())
}