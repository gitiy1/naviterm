
use ratatui::{prelude::*, widgets::*};
use crate::app::{App, CurrentScreen};
use crate::ui::popup_connection_test;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    frame.render_widget(
        Tabs::new(vec!["Home", "Albums", "Playlists", "Artists"])
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(0)
            .divider(symbols::line::VERTICAL)
            .padding(" ", " "),
        frame.size()
    );
    
    match app.current_screen {
        CurrentScreen::Home => {}
        CurrentScreen::Albums => {}
        CurrentScreen::Playlists => {}
        CurrentScreen::Artists => {}
        CurrentScreen::ConnectionTest => {popup_connection_test::draw_popup(app,frame).unwrap()}
    }

}

