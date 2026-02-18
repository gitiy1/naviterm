use crate::app::{App, AppColors, AppConnectionMode, AppStatus, CurrentScreen, Popup};
use crate::ui::footer_now_playing::draw_footer;
use crate::ui::{popup_add_to, popup_album_info, popup_connection_error, popup_connection_test, popup_deletion_playlist, popup_error_message, popup_genre_filter, popup_global_search, popup_keybindings, popup_select_playlist, popup_synchronize_playlist, popup_update, popup_year_filter, tab_albums, tab_artists, tab_home, tab_playlists, tab_queue};
use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::Line;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::Tabs;
use ratatui::{symbols, Frame};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let vertical = Layout::vertical([Length(1), Min(0), Length(4)]);
    let [header_area, inner_area, footer_area] = vertical.areas(frame.area());
    let horizontal = Layout::horizontal([Percentage(50), Percentage(50)]);
    let [tabs_area, title_area] = horizontal.areas(header_area);

    match app.current_screen {
        CurrentScreen::Home => {
            draw_tabs(&app.app_colors, 0, tabs_area, frame);
            tab_home::draw_tab(app, inner_area, frame).unwrap();
        }
        CurrentScreen::Albums => {
            draw_tabs(&app.app_colors, 1, tabs_area, frame);
            tab_albums::draw_tab(app, inner_area, frame).unwrap();
        }
        CurrentScreen::Playlists => {
            draw_tabs(&app.app_colors, 2, tabs_area, frame);
            tab_playlists::draw_tab(app, inner_area, frame).unwrap();
        }
        CurrentScreen::Artists => {
            draw_tabs(&app.app_colors, 3, tabs_area, frame);
            tab_artists::draw_tab(app, inner_area, frame).unwrap();
        }
        CurrentScreen::Queue => {
            draw_tabs(&app.app_colors, 4, tabs_area, frame);
            tab_queue::draw_tab(app, inner_area, frame).unwrap();
        }
    }

    match app.current_popup {
        Popup::ConnectionTest => popup_connection_test::draw_popup(app, frame).unwrap(),
        Popup::AlbumInformation => popup_album_info::draw_popup(app, frame).unwrap(),
        Popup::AddTo => popup_add_to::draw_popup(app, frame).unwrap(),
        Popup::GenreFilter => popup_genre_filter::draw_popup(app, frame).unwrap(),
        Popup::YearFilter => popup_year_filter::draw_popup(app, frame).unwrap(),
        Popup::UpdateDatabase => popup_update::draw_popup(app, frame).unwrap(),
        Popup::SelectPlaylist => popup_select_playlist::draw_popup(app, frame).unwrap(),
        Popup::SynchronizePlaylist => popup_synchronize_playlist::draw_popup(app, frame).unwrap(),
        Popup::ConfirmPlaylistDeletion => popup_deletion_playlist::draw_popup(app, frame).unwrap(),
        Popup::ConnectionError => popup_connection_error::draw_popup(app, frame).unwrap(),
        Popup::GlobalSearch => popup_global_search::draw_popup(app, frame).unwrap(),
        Popup::ErrorMessage => popup_error_message::draw_popup(app, frame).unwrap(),
        Popup::Keybindings => popup_keybindings::draw_popup(app, frame).unwrap(),
        Popup::None => {}
    }

    draw_title(app, title_area, frame);
    draw_footer(app, footer_area, frame);
}

fn draw_title(app: &mut App, title_area: Rect, frame: &mut Frame) {
    let horizontal = Layout::horizontal([Percentage(50), Percentage(50)]);
    let [search_area, status_area] = horizontal.areas(title_area);
    let mut search_line: Vec<Span> = vec![];
    if !app.search_data.search_string.is_empty() || app.app_flags.getting_search_string {
        search_line.push(Span::from("Searching: "));
        search_line.push(
            Span::from(app.search_data.search_string.clone()).style(
                Style::default()
                    .fg(app.app_colors.primary_accent)
                    .add_modifier(Modifier::ITALIC),
            ),
        );
    }
    if app.search_data.index_in_search != usize::MAX {
        search_line.push(
            Span::from(format!(
                " ({}/{})",
                app.search_data.index_in_search + 1,
                app.search_data.search_results_indexes.len()
            ))
            .style(
                Style::default()
                    .fg(app.app_colors.secondary_accent)
                    .add_modifier(Modifier::ITALIC),
            ),
        );
    }
    if !app.search_data.search_string.is_empty()
        && !app.app_flags.getting_search_string
        && app.search_data.search_results_indexes.is_empty()
    {
        search_line.push(
            Span::from(" (Not found)").style(
                Style::default()
                    .fg(app.app_colors.secondary_accent)
                    .add_modifier(Modifier::ITALIC),
            ),
        );
    }
    let pending_operations = app.server.operations.len().to_string();
    let status_span = if app.mode == AppConnectionMode::Offline {
        Span::from("Offline").style(
            Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        match app.status {
            AppStatus::Connected => Span::from("Connected").style(
                Style::default()
                    .fg(app.app_colors.connected)
                    .add_modifier(Modifier::BOLD),
            ),
            AppStatus::Disconnected => Span::from("Disconnected").style(
                Style::default()
                    .fg(app.app_colors.error)
                    .add_modifier(Modifier::BOLD),
            ),
            AppStatus::Updating => {
                Span::from("Updating (".to_owned() + pending_operations.as_str() + ")").style(
                    Style::default()
                        .fg(app.app_colors.updating)
                        .add_modifier(Modifier::BOLD),
                )
            }
        }
    };
    let status_line = Line::from(vec![Span::from("naviterm - "), status_span]);
    frame.render_widget(Line::from(search_line), search_area);
    frame.render_widget(status_line, status_area);
}

fn draw_tabs(app_colors: &AppColors, index: usize, header_area: Rect, frame: &mut Frame) {
    frame.render_widget(
        Tabs::new(vec!["Home", "Albums", "Playlists", "Artists", "Queue"])
            .style(Style::default().white())
            .highlight_style(Style::default().fg(app_colors.primary_accent))
            .select(index)
            .divider(symbols::line::VERTICAL)
            .padding(" ", " "),
        header_area,
    );
}
