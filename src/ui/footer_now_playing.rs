use ratatui::{Frame, symbols};
use ratatui::layout::{Layout, Rect};
use ratatui::layout::Constraint::{Length, Percentage};
use ratatui::prelude::Text;
use ratatui::style::{Modifier, Style};
use ratatui::style::Color::{DarkGray, Gray, Yellow};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, LineGauge, Padding, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::App;
use crate::player::mpv::PlayerStatus;
use crate::ui::utils::duration_to_hhmmss;

pub fn draw_footer(app: &mut App, footer_area: Rect, frame: &mut Frame) {


    let seconds_played = (app.ticks_during_playing_state / 4).to_string();
    let player_status = match app.player.player_status() {
        PlayerStatus::Playing => {
            Line::from(vec![
                Span { content: "Now playing - [".into(), style: Style::default()},
                Span { content: duration_to_hhmmss(seconds_played.as_str()).into(), style: Style::default()},
                Span { content: "] ".into(), style: Style::default()},
            ])
        }
        PlayerStatus::Paused => {Line::from("Paused")}
        PlayerStatus::Stopped => {Line::from("Stopped")}
    };
    

    let block = if app.queue_has_next() {
        Block::bordered()
            .title(player_status.left_aligned())
            .title(Line::raw("Next").right_aligned())
            .border_type(Rounded)
    }
        else {
            Block::bordered()
                .title(player_status.left_aligned())
                .border_type(Rounded)
            
        };
        

    let inner = block.inner(footer_area);
    let sections = Layout::horizontal([Percentage(30), Percentage(60), Percentage(30)]);
    let [current, status, next] = sections.areas(inner);
    let status_layout = Layout::vertical([Length(1), Length(1)]);
    let [status_text_area, status_progress_bar_area] = status_layout.areas(status);
    
    let mut ratio;
    let footer = if app.now_playing.is_empty() {
        ratio = 0f64;
        Paragraph::new("Nothing in the playing queue")
    }
    else {
        let song = app.database.get_song(app.now_playing.as_str());
        ratio = (app.ticks_during_playing_state / 4) as f64 / song.duration().parse::<usize>().unwrap() as f64 ;
        ratio = if ratio > 1f64 { 1f64 } else { ratio };
        Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span { content: song.title().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
            ]),
            Line::from(vec![
                Span { content: song.artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
            ])
        ]))
    };

    let status_block = Block::default().padding(
        Padding::new(2,2,0,0)
    );

    let progress = LineGauge::default()
        .filled_style(
            Style::default()
                .fg(Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .unfilled_style(
            Style::default()
                .fg(DarkGray)
        )
        .label("")
        .line_set(symbols::line::THICK)
        .ratio(ratio);
    
    let status_text = Line::from("random: off, repeat: off")
        .style(Style::default().add_modifier(Modifier::ITALIC)).centered();

    frame.render_widget(footer,current);
    if app.player.player_status != PlayerStatus::Stopped {
        frame.render_widget(status_block,status);
        frame.render_widget(status_text,status_text_area);
        frame.render_widget(progress,status_progress_bar_area);
    }
    if app.queue_has_next() {
        let song = app.database.get_song(app.queue.get(app.index_in_queue+1).unwrap());
        let next_song_info = Paragraph::new(Text::from(vec![
                Line::from(vec![
                    Span { content: song.title().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
                ]),
                Line::from(vec![
                    Span { content: song.artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                ])
            ])).right_aligned();
        frame.render_widget(next_song_info,next);
    }
    frame.render_widget(block,footer_area);
}
