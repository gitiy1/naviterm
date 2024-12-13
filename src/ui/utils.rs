use crate::app::{AppFlags, HomePane, SearchData};
use crate::music_database::MusicDatabase;
use log::debug;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Color::{Black, Gray, Yellow};
use ratatui::prelude::{Line, Modifier, Span, Style, Text};
use ratatui::widgets::ListItem;
use unicode_segmentation::UnicodeSegmentation;

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub fn duration_to_hhmmss(duration: &str) -> String {
    let u_duration = duration.parse::<usize>().unwrap();
    let hhmmss;

    if u_duration > 3600 {
        let hours = u_duration / 3600;
        let minutes = (u_duration - 3600 * hours) / 60;
        let seconds = (u_duration - 3600 * hours) % 60;
        if minutes < 10 && seconds < 10 {
            hhmmss = format!("{}:0{}:0{}", hours, minutes, seconds);
        } else if minutes < 10 {
            hhmmss = format!("{}:0{}:{}", hours, minutes, seconds);
        } else if seconds < 10 {
            hhmmss = format!("{}:{}:0{}", hours, minutes, seconds);
        } else {
            hhmmss = format!("{}:{}:{}", hours, minutes, seconds);
        }
    } else {
        let minutes = u_duration / 60;
        let seconds = u_duration % 60;
        if seconds < 10 {
            hhmmss = format!("{}:0{}", minutes, seconds);
        } else {
            hhmmss = format!("{}:{}", minutes, seconds);
        }
    }

    hhmmss
}

pub fn ellipse_line(line: &str, max_width: usize) -> String {
    if line.is_empty() {
        String::new()
    } else if line.graphemes(true).count() > max_width {
        let clipped_line: String = line.graphemes(true).take(max_width - 4).collect();
        clipped_line + "..."
    } else {
        String::from(line)
    }
}

pub fn get_text_for_album_item<'a>(
    database: &'a MusicDatabase,
    app_flags: &AppFlags,
    index: usize,
    album_id: &str,
    search_data: &SearchData,
    home_pane: &HomePane
) -> ListItem<'a> {
    let album = database.get_album(album_id);
    let album_name_to_search = if app_flags.upper_case_search {
        album.name().to_string()
    } else {
        album.name().to_lowercase()
    };
    let mut album_first_line_vector: Vec<Span> = vec![];
    let mut album_second_line_vector: Vec<Span> = vec![];
    if !search_data.search_results_indexes.is_empty()
        && index == *search_data.search_results_indexes.get(search_data.index_in_search).unwrap()
        && *home_pane == HomePane::TopLeft
    {
        debug!("album: {}, search string: {}, album index: {}, app search index: {}, search matches indexes: {:?}", album_name_to_search, search_data.search_string, index, search_data.index_in_search, search_data.search_results_indexes);
        let match_indices: Vec<_> = album_name_to_search
            .match_indices(&search_data.search_string)
            .collect();
        debug!("match: {:?}", match_indices);
        let (first_index, first_match) = match_indices[0];
        let first_slice = &album.name()[0..first_index];
        let matched_slice = &album.name()[first_index..first_index + first_match.len()];
        let last_slice = &album.name()[first_index + first_match.len()..];
        album_first_line_vector.push(
            Span::from(first_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
        album_first_line_vector.push(
            Span::from(matched_slice.to_string()).style(
                Style::default()
                    .fg(Black)
                    .bg(Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        album_first_line_vector.push(
            Span::from(last_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
    } else {
        album_first_line_vector.push(
            Span::from(album.name())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        )
    }
    album_second_line_vector.push(Span {
        content: "from ".into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    album_second_line_vector.push(Span {
        content: database.get_album(album_id).artist().into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    album_second_line_vector.push(Span {
        content: ", ".into(),
        style: Style::default(),
    });
    album_second_line_vector.push(Span {
        content: database.get_album(album_id).song_count().into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    album_second_line_vector.push(Span {
        content: " songs".into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });

    ListItem::from(Text::from(vec![
        Line::from(album_first_line_vector.clone()),
        Line::from(album_second_line_vector.clone()),
    ]))
}

