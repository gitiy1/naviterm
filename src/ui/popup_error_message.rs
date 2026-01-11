use crate::app::{App, AppResult};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Alignment, Line, Modifier};
use ratatui::style::Style;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(50, 30, frame.area());

    let block = Block::bordered()
        .title(Line::raw("Error Message").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let popup_footer_line = Line::from(format!(
        "{} close popup",
        app.shortcuts
            .get_key_combo_for_operation(ShortcutAction::PopupClose, None),
    ))
    .style(
        Style::default()
            .fg(app.app_colors.secondary_accent)
            .add_modifier(Modifier::ITALIC),
    )
    .alignment(Alignment::Center);

    let popup_content = Paragraph::new(app.error_message.to_string()).wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(popup_content, chunks[0]);
    frame.render_widget(popup_footer_line, chunks[1]);
    frame.render_widget(block, area);

    Ok(())
}
