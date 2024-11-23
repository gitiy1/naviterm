use crate::app::{App, AppResult, ArtistPane};
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
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    
    let mut block_artists = Block::bordered()
        .border_type(Rounded)
        .border_style(Style::default().fg(Gray));
    
    let mut block_artist_selected = Block::bordered()
        .border_type(Rounded)
        .border_style(Style::default().fg(Gray));

    let active_pane_style = Style::default().fg(Yellow);
    
    match app.artist_pane {
        ArtistPane::Left => {
            block_artists = block_artists.border_style(active_pane_style);
        }
        ArtistPane::Right => {
            block_artist_selected = block_artist_selected.border_style(active_pane_style);
        }
    }

    if app.database.artists().is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("No artists..."))
                .block(Block::bordered().border_type(Rounded)),
            area,
        );
    } else {
        let mut artists_items: Vec<ListItem> = Vec::new();
        for artist_id in app.database.alphabetical_artists() {
            let artist = app.database.get_artist(artist_id);
            let artist_item = Text::from(vec![Line::from(vec![Span {
                content: artist.name().into(),
                style: Style::default().fg(Yellow),
            }])]);
            artists_items.push(ListItem::from(artist_item));
        }
        let list = List::new(artists_items)
            .block(block_artists)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        
        if app.list_states.artist_state.selected().is_none() {
            app.list_states.artist_state.select_first()
        }

        frame.render_stateful_widget(list, chunks[0], &mut app.list_states.artist_state);

        let selected_artist = app.database.get_artist(
            app.database
                .alphabetical_artists()
                .get(app.list_states.artist_state.selected().unwrap())
                .unwrap(),
        );

        let mut album_items: Vec<ListItem> = vec![];
        for album_id in selected_artist.albums() {
            let album = app.database.get_album(album_id);
            let album_item = Text::from(vec![
                Line::from(vec![Span {
                    content: album.name().into(),
                    style: Style::default().fg(Yellow),
                }]),
                Line::from(vec![
                    Span {
                        content: duration_to_hhmmss(album.duration()).into(),
                        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
                    },
                    Span {
                        content: " - ".into(),
                        style: Style::default(),
                    },
                    Span {
                        content: album.genres().join(", ").into(),
                        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
                    },
                    Span {
                        content: " - ".into(),
                        style: Style::default(),
                    },
                    Span {
                        content: album.song_count().into(),
                        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
                    },
                    Span {
                        content: " songs".into(),
                        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
                    },
                ]),
            ]);
            album_items.push(ListItem::from(album_item));
            for song_id in album.songs() {
                let song = app.database.get_song(song_id);
                let song_item = Text::from(Line::from(vec![
                    Span {
                        content: "  ".into(),
                        style: Style::default(),
                    },
                    Span {
                        content: song.title().into(),
                        style: Style::default(),
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
                ]));
                album_items.push(ListItem::from(song_item));
            }
        }

        let list = List::new(album_items)
            .block(block_artist_selected)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);

        if app.list_states.artist_selected_state.selected().is_none() {
            app.list_states.artist_selected_state.select_first();
        }

        frame.render_stateful_widget(
            list,
            chunks[1],
            &mut app.list_states.artist_selected_state,
        );
    }

    Ok(())
}
