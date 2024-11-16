use crate::app::{App, AppResult};
use crate::ui::utils::duration_to_hhmmss;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    if app.database.playlists().is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No playlists..."))
                .block(Block::bordered().border_type(Rounded)),
            area,
        );
    } else {
        let mut playlist_items: Vec<ListItem> = Vec::new();
        for playlist in app.database.playlists() {
            let playlist_item = Text::from(vec![Line::from(vec![Span {
                content: playlist.name().into(),
                style: Style::default().fg(Yellow),
            }])]);
            playlist_items.push(ListItem::from(playlist_item));
        }
        let list = List::new(playlist_items)
            .block(Block::bordered().border_type(Rounded))
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        if app.list_states.playlist_state.selected().is_none() {
            app.list_states.playlist_state.select_first()
        }

        frame.render_stateful_widget(list, chunks[0], &mut app.list_states.playlist_state);

        let song_items = app
            .database
            .playlists()
            .get(app.list_states.playlist_state.selected().unwrap())
            .unwrap()
            .song_list()
            .iter()
            .enumerate()
            .map(|(_i, song_id)| {
                let song = app.database.get_song(song_id);
                let song_item = Text::from(vec![Line::from(vec![
                    Span {
                        content: song.title().into(),
                        style: Style::default().fg(Yellow),
                    },
                    Span {
                        content: " (".into(),
                        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
                    },
                    Span {
                        content: duration_to_hhmmss(song.duration()).into(),
                        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
                    },
                    Span {
                        content: ")".into(),
                        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
                    },
                ])]);
                ListItem::from(song_item)
            });
        let list = List::new(song_items)
            .block(Block::bordered().border_type(Rounded))
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        if app.list_states.playlist_selected_state.selected().is_none() {
            app.list_states.playlist_selected_state.select_first()
        }

        frame.render_stateful_widget(list, chunks[1], &mut app.list_states.playlist_selected_state);
    }

    Ok(())
}
