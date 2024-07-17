
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::{Frame, symbols};
use ratatui::prelude::Line;
use ratatui::widgets::{Block, Paragraph, Tabs};
use crate::app::{App, Popup, CurrentScreen};
use crate::ui::{popup_album_info, popup_connection_test, tab_home};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {

    let vertical = Layout::vertical([Length(1), Min(0), Length(4)]);
    let [header_area, inner_area, footer_area] = vertical.areas(frame.size());
    let horizontal = Layout::horizontal([Min(0), Length(20)]);
    let [tabs_area, title_area] = horizontal.areas(header_area);


    match app.current_screen {
        CurrentScreen::Home => {
            draw_tabs(0,tabs_area, frame);
            tab_home::draw_tab(app, inner_area, frame).unwrap();
        }
        CurrentScreen::Albums => {}
        CurrentScreen::Playlists => {}
        CurrentScreen::Artists => {}
    }
    
    match app.current_popup {
        Popup::ConnectionTest => {popup_connection_test::draw_popup(app, frame).unwrap()}
        Popup::AlbumInformation => {popup_album_info::draw_popup(app, frame).unwrap()}
        Popup::None => {}
    }

    draw_title(title_area, frame);

}

fn draw_title(title_area: Rect, frame: &mut Frame) {
    frame.render_widget(Paragraph::new(
        Line::from("naviterm"))
        .block(Block::default())
        ,title_area);
}

fn draw_tabs(index: usize, header_area: Rect, frame: &mut Frame) {

    frame.render_widget(
        Tabs::new(vec!["Home", "Albums", "Playlists", "Artists"])
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(index)
            .divider(symbols::line::VERTICAL)
            .padding(" ", " "),
        header_area
    );
    
}

