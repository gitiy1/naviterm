use crate::app::{App, AppResult};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Modifier, Span, Style};
use ratatui::text::Line;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Clear, Padding, Paragraph};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(50, 30, frame.size());

    let block = Block::bordered()
        .title(Line::raw("Year filter").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let (from_style, to_style) = if app.app_flags.is_introducing_to_year {
        (
            Style::default().fg(app.app_colors.secondary_accent),
            Style::default().fg(app.app_colors.primary_accent),
        )
    } else {
        (
            Style::default().fg(app.app_colors.primary_accent),
            Style::default().fg(app.app_colors.secondary_accent),
        )
    };

    let mut filter_data: Vec<Line> = vec![Line::from("Input the years to filter albums:\n")];

    if app.app_flags.range_year_filter {
        filter_data.append(&mut vec![
            Line::from(vec![
                Span {
                    content: "From: ".into(),
                    style: from_style,
                },
                Span {
                    content: app.album_filters.year_from_filter_new.clone().into(),
                    style: Style::default(),
                },
            ]),
            Line::from(vec![
                Span {
                    content: "To: ".into(),
                    style: to_style,
                },
                Span {
                    content: app.album_filters.year_to_filter_new.clone().into(),
                    style: Style::default(),
                },
            ]),
        ])
    } else {
        filter_data.append(&mut vec![Line::from(vec![
            Span {
                content: "From: ".into(),
                style: Style::default().fg(app.app_colors.primary_accent),
            },
            Span {
                content: app.album_filters.year_from_filter_new.clone().into(),
                style: Style::default(),
            },
        ])])
    };

    if !app.album_filters.filter_message.is_empty() {
        filter_data.push(
            Line::from(app.album_filters.filter_message.clone())
                .style(Style::default().fg(app.app_colors.error)),
        );
    }

    let popup_paragraph = Paragraph::new(filter_data);
    let popup_footer = if app.app_flags.range_year_filter {
        Paragraph::new(
            Line::from(format!(
                "{} accept, {} input single year, {} toggle input field",
                app.shortcuts
                    .get_key_combo_for_operation(ShortcutAction::PopupYearAcceptFilter, None),
                app.shortcuts
                    .get_key_combo_for_operation(ShortcutAction::PopupYearToggleRangeInput, None),
                app.shortcuts
                    .get_key_combo_for_operation(ShortcutAction::PopupYearToggleFromTo, None)
            ))
            .style(
                Style::default()
                    .fg(app.app_colors.secondary_accent)
                    .add_modifier(Modifier::ITALIC),
            ),
        )
    } else {
        Paragraph::new(
            Line::from(format!(
                "{} accept, {} input year range",
                app.shortcuts
                    .get_key_combo_for_operation(ShortcutAction::PopupYearAcceptFilter, None),
                app.shortcuts
                    .get_key_combo_for_operation(ShortcutAction::PopupYearToggleRangeInput, None),
            ))
            .style(
                Style::default()
                    .fg(app.app_colors.secondary_accent)
                    .add_modifier(Modifier::ITALIC),
            ),
        )
    };

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(Clear, area);
    frame.render_widget(popup_paragraph, chunks[0]);
    frame.render_widget(popup_footer, chunks[1]);
    frame.render_widget(block, area);
    Ok(())
}
