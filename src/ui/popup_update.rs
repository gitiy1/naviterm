use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;
use ratatui::prelude::{Line, Span};
use crate::app::{App, AppResult};
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(40, 30, frame.size());
    
    let mut popup_lines = vec![
        Line::from(vec![
            Span{
                content: "(b)".into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " All albums".into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span{
                content: "(y)".into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Playlists".into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span{
                content: "(s)".into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Everything - Quick scan".into(),
                style: Style::default(),
            },
        ]),
        Line::from(vec![
            Span{
                content: "(a)".into(),
                style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
            },
            Span{
                content: " Everything - Full scan".into(),
                style: Style::default(),
            },
        ]),
    ];
    
    if !app.selected_album_id_to_update.is_empty() {
        popup_lines.push(
            Line::from(vec![
                Span{
                    content: "(CR)".into(),
                    style: Style::default().fg(app.app_colors.primary_accent).add_modifier(Modifier::BOLD),
                },
                Span{
                    content: " Currently selected: ".into(),
                    style: Style::default(),
                },
                Span{
                    content: app.database.get_album(app.selected_album_id_to_update.as_str()).name().into(),
                    style: Style::default(),
                },
            ]),
        );
    }
    
    let popup_block = Paragraph::new(popup_lines)
    .wrap(Wrap { trim: true })
    .block(
        Block::bordered()
            .title("Update database")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(4, 4, 1, 1)),
    )
    .style(Style::default().fg(Color::default()).bg(Color::default()));

    frame.render_widget(Clear, area);
    frame.render_widget(popup_block, area);

    Ok(())
}
