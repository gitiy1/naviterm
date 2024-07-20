use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Text;
use ratatui::style::{Modifier, Style};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::App;

pub fn draw_footer(app: &mut App, footer_area: Rect, frame: &mut Frame) {


    let block = Block::bordered()
        .title(Line::raw("Now playing").left_aligned())
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
            ])
        ]))
    };

    frame.render_widget(footer.block(block),footer_area);
}
