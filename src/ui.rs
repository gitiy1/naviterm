//use ratatui::{
//    layout::Alignment,
//    style::{Color, Style},
//    widgets::{Block, Tabs, BorderType, Paragraph},
//    Frame,
//};
use ratatui::{prelude::*, widgets::*};
use crate::app::{App, CurrentScreen};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    //frame.render_widget(
    //    Paragraph::new(format!(
    //        "This is a tui template.\n\
    //            Salt: {}, token: {}.\n\
    //            Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
    //            Press left and right to increment and decrement the counter respectively.\n\
    //            Counter: {}",
    //        app.get_salt().unwrap(),
    //        app.get_token().unwrap(),
    //        app.counter
    //    ))
    //    .block(
    //        Block::bordered()
    //            .title("Template")
    //            .title_alignment(Alignment::Center)
    //            .border_type(BorderType::Rounded),
    //    )
    //    .style(Style::default().fg(Color::Cyan))
    //    .centered(),
    //    frame.size(),
    //)
    frame.render_widget(
    Tabs::new(vec!["Home", "Albums", "Playlists", "Artists"])
        .style(Style::default().white())
        .highlight_style(Style::default().yellow())
        .select(0)
        .divider(symbols::line::VERTICAL)
        .padding(" ", " "),
        frame.size()
    );

    if let CurrentScreen::ConnectionTest = app.current_screen {
        let area = centered_rect(60, 40, frame.size());

        let chunks = Layout::default().direction(Direction::Vertical).constraints([
                Constraint::Min(1),
                Constraint::Length(3),
            ]).split(area);

        let popup_block = Paragraph::new(format!(
            "Testing details:\n\
                Salt: {}\n\
                Token: {}\n\
                Server address: {}\n\
                Connection status: {}\n\
                Last connection: {}\n\
                Connection error code: {}\n\
                Connection message: {}\n\
                ",
            app.server.salt,
            app.server.token,
            app.server.server_address,
            app.server.connection_status,
            app.server.last_connection_timestamp,
            app.server.connection_code,
            app.server.connection_message
        )).wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title("Test Navidrome server")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::default()));

        let popup_footer = Paragraph::new(Line::from("(r) to generate new salt and token (t) to test connection"))
            .block(Block::default());

        frame.render_widget(popup_block, chunks[0]);
        frame.render_widget(popup_footer, chunks[1]);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
