use crate::app::{App, AppResult};
use crate::mappings::ShortcutAction;
use crate::ui::utils::{duration_to_hhmmss, get_text_for_song_item_queue};
use ratatui::layout::Constraint::Length;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Constraint::Max;
use ratatui::prelude::Style;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Padding, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let queue_block = Block::bordered()
        .title(Line::raw("Queue").left_aligned())
        .border_type(Rounded)
        .style(Style::default());

    if app.player_data.queue.is_empty() {
        frame.render_widget(
            Paragraph::new(
                Line::from("\nNothing in the queue...")
                    .style(Style::default().fg(app.app_colors.secondary_accent)),
            )
            .alignment(Alignment::Center)
            .block(queue_block),
            area,
        );
    } else {
        let queue_block_inner = queue_block.inner(chunks[0]);

        let chunks_queue = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Percentage(100)])
            .split(queue_block_inner);

        let seconds_left = app
            .player_data
            .duration_left
            .parse::<usize>()
            .unwrap()
            .saturating_sub(app.ticks_during_playing_state / 4);

        let queue_info = Paragraph::new(
            Line::from(format!(
                "Total duration: {} - Playing song {}/{} ({} left)",
                duration_to_hhmmss(&app.player_data.duration_total),
                app.player_data.index_in_queue + 1,
                app.player_data.queue.len(),
                duration_to_hhmmss(seconds_left.to_string().as_str())
            ))
            .style(Style::default().fg(app.app_colors.secondary_accent)),
        )
        .alignment(Alignment::Center);

        frame.render_widget(queue_info, chunks_queue[0]);

        let mut items: Vec<ListItem> = Vec::new();
        let mut reordered_queue: Vec<String> = vec![];
        let iterator = if app.app_config.reorder_random_queue {
            reordered_queue = app.player_data.queue_order.iter().map(|i| {app.player_data.queue[*i].clone()}).collect::<Vec<String>>();
            reordered_queue.iter()
        } else {
            app.player_data.queue.iter()
        };
        for (index, song_id) in iterator.enumerate() {
            items.push(get_text_for_song_item_queue(
                &app.database,
                &mut app.app_flags,
                &app.app_colors,
                app.list_states.queue_list_state.selected().unwrap(),
                index,
                song_id,
                &app.search_data,
                &app.player_data.now_playing.id,
            ));
        }
        let list = List::new(items)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        if app.app_flags.move_to_next_in_search {
            app.app_flags.move_to_next_in_search = false;
            app.list_states.queue_list_state.select(Some(
                *app.search_data
                    .search_results_indexes
                    .get(app.search_data.index_in_search)
                    .unwrap(),
            ));
        }
        frame.render_stateful_widget(list, chunks_queue[1], &mut app.list_states.queue_list_state);
        frame.render_widget(queue_block, chunks[0]);

        let info_block = Block::bordered()
            .title(Line::raw("Song info").left_aligned())
            .border_type(Rounded)
            .style(Style::default())
            .padding(Padding::new(2, 2, 1, 1));

        let info_block_inner = info_block.inner(chunks[1]);

        let vertical = Layout::vertical([Max(100), Length(3)]);
        let [info_area, navigation_area] = vertical.areas(info_block_inner);

        let current_song_id = if app.app_config.reorder_random_queue {
            reordered_queue.get(app.list_states.queue_list_state.selected().unwrap()).unwrap()
        }
        else {
                app.player_data
                    .queue
                    .get(app.list_states.queue_list_state.selected().unwrap())
                    .unwrap()
        };
        let current_song = app.database.get_song(current_song_id);

        let song_information = Paragraph::new(vec![
            Line::from(vec![
                Span {
                    content: "Title: ".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: current_song.title().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Artist: ".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: current_song.artist().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Album: ".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: current_song.album().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Genres: ".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: current_song.genres().join(", ").into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Duration: ".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: duration_to_hhmmss(current_song.duration()).into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Play count: ".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: current_song.play_count().into(),
                    style: Style::default(),
                },
            ]),
            Line::from("\n"),
            Line::from(vec![
                Span {
                    content: "Bit rate: ".into(),
                    style: Style::default().fg(app.app_colors.secondary_accent),
                },
                Span {
                    content: current_song.bit_rate().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Track peak: ".into(),
                    style: Style::default().fg(app.app_colors.secondary_accent),
                },
                Span {
                    content: current_song.track_peak().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Track gain: ".into(),
                    style: Style::default().fg(app.app_colors.secondary_accent),
                },
                Span {
                    content: current_song.track_gain().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Album peak: ".into(),
                    style: Style::default().fg(app.app_colors.secondary_accent),
                },
                Span {
                    content: current_song.album_peak().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "Album gain: ".into(),
                    style: Style::default().fg(app.app_colors.secondary_accent),
                },
                Span {
                    content: current_song.album_gain().into(),
                    style: Style::default(),
                },
            ]),
        ])
        .wrap(Wrap { trim: true });

        frame.render_widget(song_information, info_area);

        let navigation_options = Paragraph::new(vec![
            Line::from(vec![
                Span {
                    content: app
                        .shortcuts
                        .get_key_combo_for_operation(ShortcutAction::GoToTrackAlbum, None)
                        .into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: " Go to album in albums pane".into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: app
                        .shortcuts
                        .get_key_combo_for_operation(ShortcutAction::GoToTrackArtist, None)
                        .into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: " Go to artist in artists pane".into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: app
                        .shortcuts
                        .get_key_combo_for_operation(ShortcutAction::QueueDeleteSong, None)
                        .into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: " Delete song from queue".into(),
                    style: Style::default(),
                },
            ]),
        ]);

        frame.render_widget(navigation_options, navigation_area);
        frame.render_widget(info_block, chunks[1]);
    }

    Ok(())
}
