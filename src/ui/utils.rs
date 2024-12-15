use crate::app::{AppFlags, HomePane, SearchData};
use crate::music_database::MusicDatabase;
use log::debug;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Color::{Black, Gray, Green, Yellow};
use ratatui::prelude::{Line, Modifier, Span, Style, Stylize, Text};
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
    home_pane: &HomePane,
    current_pane: &HomePane,
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
        && index
            == *search_data
                .search_results_indexes
                .get(search_data.index_in_search)
                .unwrap()
        && *home_pane == *current_pane
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

pub fn get_text_for_song_item_queue<'a>(
    database: &'a MusicDatabase,
    app_flags: &AppFlags,
    index: usize,
    song_id: &str,
    search_data: &SearchData,
    queue_order: &[usize],
    index_in_queue: usize,
) -> ListItem<'a> {
    let song = database.get_song(song_id);
    let mut song_first_line_vector: Vec<Span> = vec![];
    let mut song_second_line_vector: Vec<Span> = vec![];
    let style_playing = if index == *queue_order.get(index_in_queue).unwrap() {
        Style::default().fg(Green)
    } else {
        Style::default().fg(Yellow)
    };
    if !search_data.search_results_indexes.is_empty()
        && index
            == *search_data
                .search_results_indexes
                .get(search_data.index_in_search)
                .unwrap()
    {
        let song_name_to_search = if app_flags.upper_case_search {
            song.title().to_string()
        } else {
            song.title().to_lowercase()
        };
        debug!("song: {}, search string: {}, song index: {}, app search index: {}, search matches indexes: {:?}", song_name_to_search, search_data.search_string, index, search_data.index_in_search, search_data.search_results_indexes);
        let match_indices: Vec<_> = song_name_to_search
            .match_indices(&search_data.search_string)
            .collect();
        debug!("match: {:?}", match_indices);
        let (first_index, first_match) = match_indices[0];
        let first_slice = &song.title()[0..first_index];
        let matched_slice = &song.title()[first_index..first_index + first_match.len()];
        let last_slice = &song.title()[first_index + first_match.len()..];

        song_first_line_vector.push(
            Span::from(first_slice.to_string())
                .style(style_playing).add_modifier(Modifier::BOLD),
        );
        song_first_line_vector.push(
            Span::from(matched_slice.to_string()).style(
                Style::default()
                    .fg(Black)
                    .bg(Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        song_first_line_vector.push(
            Span::from(last_slice.to_string())
                .style(style_playing).add_modifier(Modifier::BOLD),
        );
    } else {
        song_first_line_vector.push(
            Span::from(song.title())
                .style(style_playing).add_modifier(Modifier::BOLD),
        )
    }
    song_second_line_vector.push(Span {
        content: duration_to_hhmmss(song.duration()).into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    song_second_line_vector.push(Span {
        content: " - played ".into(),
        style: Style::default().fg(Gray),
    });
    song_second_line_vector.push(Span {
        content: song.play_count().into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    song_second_line_vector.push(Span {
        content: " times, by ".into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    song_second_line_vector.push(Span {
        content: song.artist().into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });

    ListItem::from(Text::from(vec![
        Line::from(song_first_line_vector.clone()),
        Line::from(song_second_line_vector.clone()),
    ]))
}

pub fn get_text_for_song_item<'a>(
    database: &'a MusicDatabase,
    app_flags: &AppFlags,
    index: usize,
    song_id: &str,
    search_data: &SearchData,
    searched_pane: u8,
    current_pane: u8,
) -> ListItem<'a> {
    let song = database.get_song(song_id);
    let mut song_first_line_vector: Vec<Span> = vec![];
    let mut song_second_line_vector: Vec<Span> = vec![];
    if !search_data.search_results_indexes.is_empty()
        && index
        == *search_data
        .search_results_indexes
        .get(search_data.index_in_search)
        .unwrap()
        && searched_pane == current_pane
    {
        let song_name_to_search = if app_flags.upper_case_search {
            song.title().to_string()
        } else {
            song.title().to_lowercase()
        };
        debug!("song: {}, search string: {}, song index: {}, app search index: {}, search matches indexes: {:?}", song_name_to_search, search_data.search_string, index, search_data.index_in_search, search_data.search_results_indexes);
        let match_indices: Vec<_> = song_name_to_search
            .match_indices(&search_data.search_string)
            .collect();
        debug!("match: {:?}", match_indices);
        let (first_index, first_match) = match_indices[0];
        let first_slice = &song.title()[0..first_index];
        let matched_slice = &song.title()[first_index..first_index + first_match.len()];
        let last_slice = &song.title()[first_index + first_match.len()..];
        song_first_line_vector.push(
            Span::from(first_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
        song_first_line_vector.push(
            Span::from(matched_slice.to_string()).style(
                Style::default()
                    .fg(Black)
                    .bg(Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        song_first_line_vector.push(
            Span::from(last_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
    } else {
        song_first_line_vector.push(
            Span::from(song.title())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        )
    }
    song_second_line_vector.push(Span {
        content: duration_to_hhmmss(song.duration()).into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    song_second_line_vector.push(Span {
        content: " - played ".into(),
        style: Style::default().fg(Gray),
    });
    song_second_line_vector.push(Span {
        content: song.play_count().into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    song_second_line_vector.push(Span {
        content: " times, by ".into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });
    song_second_line_vector.push(Span {
        content: song.artist().into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });

    ListItem::from(Text::from(vec![
        Line::from(song_first_line_vector.clone()),
        Line::from(song_second_line_vector.clone()),
    ]))
}

pub fn get_text_for_playlist_item<'a>(
    database: &'a MusicDatabase,
    app_flags: &AppFlags,
    index: usize,
    playlist_id: &str,
    search_data: &SearchData,
    searched_pane: u8,
    current_pane: u8,
) -> ListItem<'a> {
    let playlist = database.playlists().get(playlist_id).unwrap();
    let playlist_name_to_search = if app_flags.upper_case_search {
        playlist.name().to_string()
    } else {
        playlist.name().to_lowercase()
    };
    let mut playlist_first_line_vector: Vec<Span> = vec![];

    let modified_indicator = if playlist.modified() {
        " - Modified"
    } else if playlist.id().starts_with("local") {
        " - Local"
    } else {
        ""
    };

    if !search_data.search_results_indexes.is_empty()
        && index
            == *search_data
                .search_results_indexes
                .get(search_data.index_in_search)
                .unwrap()
        && searched_pane == current_pane
    {
        debug!("playlist: {}, search string: {}, song index: {}, app search index: {}, search matches indexes: {:?}", playlist_name_to_search, search_data.search_string, index, search_data.index_in_search, search_data.search_results_indexes);
        let match_indices: Vec<_> = playlist_name_to_search
            .match_indices(&search_data.search_string)
            .collect();
        debug!("match: {:?}", match_indices);
        let (first_index, first_match) = match_indices[0];
        let first_slice = &playlist.name()[0..first_index];
        let matched_slice = &playlist.name()[first_index..first_index + first_match.len()];
        let last_slice = &playlist.name()[first_index + first_match.len()..];

        playlist_first_line_vector.push(
            Span::from(first_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
        playlist_first_line_vector.push(
            Span::from(matched_slice.to_string()).style(
                Style::default()
                    .fg(Black)
                    .bg(Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        playlist_first_line_vector.push(
            Span::from(last_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
    } else {
        playlist_first_line_vector.push(Span {
            content: playlist.name().into(),
            style: Style::default().fg(Yellow),
        });
    }
    
    playlist_first_line_vector.push(Span {
        content: modified_indicator.into(),
        style: Style::default().fg(Gray).add_modifier(Modifier::ITALIC),
    });

    ListItem::from(Text::from(vec![Line::from(
        playlist_first_line_vector.clone(),
    )]))
}

pub fn get_text_for_artist_item<'a>(
    database: &'a MusicDatabase,
    app_flags: &AppFlags,
    index: usize,
    artist_id: &str,
    search_data: &SearchData,
    searched_pane: u8,
    current_pane: u8,
) -> ListItem<'a> {
    let artist = database.get_artist(artist_id);
    let mut artist_first_line_vector: Vec<Span> = vec![];

    if !search_data.search_results_indexes.is_empty()
        && index
        == *search_data
        .search_results_indexes
        .get(search_data.index_in_search)
        .unwrap()
        && searched_pane == current_pane
    {
        let artist_name_to_search = if app_flags.upper_case_search {
            artist.name().to_string()
        } else {
            artist.name().to_lowercase()
        };
        debug!("playlist: {}, search string: {}, song index: {}, app search index: {}, search matches indexes: {:?}", artist_name_to_search, search_data.search_string, index, search_data.index_in_search, search_data.search_results_indexes);
        let match_indices: Vec<_> = artist_name_to_search
            .match_indices(&search_data.search_string)
            .collect();
        debug!("match: {:?}", match_indices);
        let (first_index, first_match) = match_indices[0];
        let first_slice = &artist.name()[0..first_index];
        let matched_slice = &artist.name()[first_index..first_index + first_match.len()];
        let last_slice = &artist.name()[first_index + first_match.len()..];
        
        artist_first_line_vector.push(
            Span::from(first_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
        artist_first_line_vector.push(
            Span::from(matched_slice.to_string()).style(
                Style::default()
                    .fg(Black)
                    .bg(Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        artist_first_line_vector.push(
            Span::from(last_slice.to_string())
                .style(Style::default().fg(Yellow).add_modifier(Modifier::BOLD)),
        );
    } else { 
        artist_first_line_vector.push(Span {
            content: artist.name().into(),
            style: Style::default().fg(Yellow),
        })
    }

    ListItem::from(Text::from(vec![Line::from(
        artist_first_line_vector.clone(),
    )]))
}
