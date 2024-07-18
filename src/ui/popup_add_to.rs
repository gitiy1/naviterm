use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, Padding, Paragraph, Wrap};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult};
use crate::ui::utils;
use crate::ui::utils::duration_to_hhmmss;

pub fn draw_popup(app: &mut App,  frame: &mut Frame) -> AppResult<()> {

    let area = utils::centered_rect(40, 30, frame.size());

    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Min(5),
        Constraint::Length(1),
    ]).split(area);

    let popup_block = Paragraph::new(format!(
        "Adding {} to...\n\n\
                (n) Queue, after current song\n\
                (e) Queue, at the end\n\
                (p) Playlist...\n\
                ", app.item_to_be_added.name
    )).wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("Add item to")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .padding(Padding::new(4, 4, 1, 1))
        )
        .style(Style::default().fg(Color::default()).bg(Color::default()));

    let popup_footer = Paragraph::new(Line::from("(n) add next (e) add at the end (p) add to playlist"))
        .block(Block::default());

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, chunks[0]);
    frame.render_widget(popup_footer, chunks[1]);


    Ok(())
}