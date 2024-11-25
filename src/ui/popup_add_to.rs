use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, AppResult, MediaType};
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    let added_item_name = match app.item_to_be_added.media_type  {
        MediaType::Song => {
            format!("track {}", app.item_to_be_added.name)
        }
        MediaType::Album => {
            format!("album {}", app.item_to_be_added.name)
        }
        MediaType::Playlist => {
            format!("playlist {}", app.item_to_be_added.name)
        }
        MediaType::Artist => {
            format!("albums from {}", app.item_to_be_added.name)
        }
    };

    let popup_block = Paragraph::new(format!(
        "Adding {} to...\n\n\
                (n) Queue, after current song\n\
                (e) Queue, at the end\n\
                (p) Playlist...\n\
                ",
        added_item_name
    ))
    .wrap(Wrap { trim: true })
    .block(
        Block::bordered()
            .title("Add item to")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(4, 4, 1, 1)),
    )
    .style(Style::default().fg(Color::default()).bg(Color::default()));

    let popup_footer = Paragraph::new(Line::from(
        "(n) add next (e) add at the end (p) add to playlist",
    ))
    .block(Block::default());

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, chunks[0]);
    frame.render_widget(popup_footer, chunks[1]);

    Ok(())
}
