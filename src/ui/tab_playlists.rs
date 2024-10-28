use crate::app::{App, AppResult};
use crate::ui::utils::duration_to_hhmmss;
use ratatui::layout::Rect;
use ratatui::style::Color::{Gray, Yellow};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let block = Block::bordered().border_type(Rounded);

    if app.database.playlists().is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No playlists..."))
                .block(Block::default())
                .block(block),
            area,
        );
    } else {
        let items = app
            .database
            .playlists()
            .iter()
            .enumerate()
            .map(|(i, playlist)| {
                let playlist_item = Text::from(vec![Line::from(vec![Span {
                    content: playlist.name().into(),
                    style: Style::default().fg(Yellow),
                }])]);
                ListItem::from(playlist_item)
            });
        let list = List::new(items)
            .block(block)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        if app.playlist_state.selected().is_none() {
            app.playlist_state.select_first()
        }
        frame.render_stateful_widget(list, area, &mut app.playlist_state);
    }

    Ok(())
}
