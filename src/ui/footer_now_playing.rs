use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Text;
use ratatui::style::{Modifier, Style};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::App;
use crate::player::mpv::PlayerStatus;
use crate::ui::utils::duration_to_hhmmss;

pub fn draw_footer(app: &mut App, footer_area: Rect, frame: &mut Frame) {
    

    let player_status = match app.player.player_status() {
        PlayerStatus::Playing => {"Now playing"}
        PlayerStatus::Paused => {"Paused"}
        PlayerStatus::Stopped => {"Stopped"}
    };
    
    let seconds_played = (app.ticks_during_playing_state / 4).to_string();

    let block = Block::bordered()
        .title(Line::raw(player_status).left_aligned())
        .border_type(Rounded)
        ;

    let footer = if app.now_playing.is_empty() {
        Paragraph::new("Nothing in the playing queue")
    }
    else {
        let song = app.database.get_song(app.now_playing.as_str());
        Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span { content: song.title().into(), style: Style::default().fg(Yellow).add_modifier(Modifier::BOLD) },
            ]),
            Line::from(vec![
                Span { content: song.artist().into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                Span { content: " - ".into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
                Span { content: duration_to_hhmmss(seconds_played.as_str()).into(), style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC) },
            ])
        ]))
    };

    frame.render_widget(footer.block(block),footer_area);
}
