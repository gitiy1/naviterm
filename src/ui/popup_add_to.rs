use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::style::Color::{Gray, Yellow};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, List, ListItem, Padding, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult};
use crate::ui::utils;
use crate::ui::utils::duration_to_hhmmss;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {

    let area = utils::centered_rect(40, 30, frame.size());

    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Min(5),
        Constraint::Length(1),
    ]).split(area);


    Ok(())
}