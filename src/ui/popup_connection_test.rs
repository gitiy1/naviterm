use crate::app::{App, AppResult};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::Span;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;
use crate::constants::NAVITERM_VERSION;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(60, 40, frame.size());

    let popup_paragraph = Paragraph::new(vec![
        Line::from(vec![
            Span {
                content: "Naviterm version: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: NAVITERM_VERSION.into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: "Salt: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: app.server.salt.clone().into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: "Token: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: app.server.token.clone().into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: "Server address: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: app.server.server_address.clone().into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: "Connection status: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: app.server.connection_status.clone().into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: "Last connection: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: app.server.last_connection_timestamp.clone().into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: "Connection error code: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: app.server.connection_code.clone().into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span {
                content: "Connection message: ".into(),
                style: Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::BOLD),
            },
            Span {
                content: app.server.connection_message.clone().into(),
                style: Style::default(),
            },
        ]),
    ])
    .wrap(Wrap { trim: true });

    let popup_footer = Paragraph::new(
        Line::from(format!(
            "{} to generate new salt and token {} to test connection",
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupTestConnectionGenerate, None),
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupTestConnectionTest, None)
        ))
        .style(
            Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        ),
    )
    .centered();

    let block = Block::bordered()
        .title("Test Navidrome server")
        .title_alignment(Alignment::Center)
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(BorderType::Rounded);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(block.inner(area));

    frame.render_widget(Clear, area);
    frame.render_widget(popup_paragraph, chunks[0]);
    frame.render_widget(popup_footer, chunks[1]);
    frame.render_widget(block, area);
    Ok(())
}
