use ratatui::layout::Constraint::{Length, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::Text;
use ratatui::style::Color::{DarkGray, Gray, Yellow};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, LineGauge, Padding, Paragraph};
use ratatui::{symbols, Frame};

use crate::app::App;
use crate::player::mpv::PlayerStatus;
use crate::ui::utils::{duration_to_hhmmss, ellipse_line};

pub fn draw_footer(app: &mut App, footer_area: Rect, frame: &mut Frame) {
    let seconds_played = (app.ticks_during_playing_state / 4).to_string();
    let player_status = match app.player.player_status() {
        PlayerStatus::Playing => Line::from(vec![
            Span {
                content: "Now playing - [".into(),
                style: Style::default(),
            },
            Span {
                content: duration_to_hhmmss(seconds_played.as_str()).into(),
                style: Style::default(),
            },
            Span {
                content: "/".into(),
                style: Style::default(),
            },
            Span {
                content: duration_to_hhmmss(
                    app.database
                        .get_song(app.now_playing.id.as_str())
                        .duration(),
                )
                .into(),
                style: Style::default(),
            },
            Span {
                content: "] ".into(),
                style: Style::default(),
            },
        ]),
        PlayerStatus::Paused => Line::from(vec![
            Span {
                content: "Paused - [".into(),
                style: Style::default(),
            },
            Span {
                content: duration_to_hhmmss(seconds_played.as_str()).into(),
                style: Style::default(),
            },
            Span {
                content: "] ".into(),
                style: Style::default(),
            },
        ]),
        PlayerStatus::Stopped => Line::from("Stopped"),
    };

    let footer_block = if app.queue_has_next() {
        Block::bordered()
            .title(player_status.left_aligned())
            .title(Line::raw("Next").right_aligned())
            .border_type(Rounded)
    } else {
        Block::bordered()
            .title(player_status.left_aligned())
            .border_type(Rounded)
    };

    let footer_inner = footer_block.inner(footer_area);
    let footer_sections = Layout::horizontal([Percentage(30), Percentage(60), Percentage(30)]);
    let [current_song_section, status_section, next_song_section] =
        footer_sections.areas(footer_inner);
    let status_block = Block::default().padding(Padding::new(2, 2, 0, 0));
    let inner_status = status_block.inner(status_section);
    let status_layout = Layout::vertical([Length(1), Length(1)]);
    let [status_text_area, status_progress_bar_area] = status_layout.areas(inner_status);
    let max_width = current_song_section.width as usize;

    let mut ratio;
    let current_song_info = if app.now_playing.id.is_empty() {
        ratio = 0f64;
        Paragraph::new("Nothing in the playing queue")
    } else {
        let song = app.database.get_song(app.now_playing.id.as_str());
        ratio = (app.ticks_during_playing_state / 4) as f64
            / song.duration().parse::<usize>().unwrap() as f64;
        ratio = if ratio > 1f64 { 1f64 } else { ratio };
        Paragraph::new(Text::from(vec![
            Line::from(Span {
                content: ellipse_line(song.title(), max_width).into(),
                style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD),
            }),
            Line::from(Span {
                content: ellipse_line(song.artist(), max_width).into(),
                style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
            }),
        ]))
    };

    let progress = LineGauge::default()
        .filled_style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD))
        .unfilled_style(Style::default().fg(DarkGray))
        .label("")
        .line_set(symbols::line::THICK)
        .ratio(ratio);

    let random_status = if app.random_playback {
        Span {
            content: "on".into(),
            style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD),
        }
    } else {
        Span {
            content: "off".into(),
            style: Style::default(),
        }
    };

    let status_text = Line::from(vec![
        Span {
            content: "random: ".into(),
            style: Style::default(),
        },
        random_status,
        Span {
            content: ", repeat: off".into(),
            style: Style::default(),
        },
    ])
    .style(Style::default().add_modifier(Modifier::ITALIC))
    .centered();

    frame.render_widget(current_song_info, current_song_section);
    if app.player.player_status != PlayerStatus::Stopped {
        frame.render_widget(status_block, status_section);
        frame.render_widget(status_text, status_text_area);
        frame.render_widget(progress, status_progress_bar_area);
    }
    if app.queue_has_next() {
        let next_song_id = app
            .queue
            .get(*app.queue_order.get(app.index_in_queue + 1).unwrap())
            .unwrap();
        let song = app.database.get_song(next_song_id);
        let next_song_info = Paragraph::new(Text::from(vec![
            Line::from(Span {
                content: ellipse_line(song.title(), max_width).into(),
                style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD),
            }),
            Line::from(Span {
                content: ellipse_line(song.artist(), max_width).into(),
                style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
            }),
        ]))
        .right_aligned();
        frame.render_widget(next_song_info, next_song_section);
    }
    frame.render_widget(footer_block, footer_area);
}
