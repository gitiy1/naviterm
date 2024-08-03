use ratatui::layout::{Constraint, Direction, Layout, Rect};
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
        let minutes = (u_duration - 3600*hours) / 60;
        let seconds = (u_duration - 3600*hours) % 60;
        if minutes < 10 && seconds < 10 {
            hhmmss = format!("{}:0{}:0{}", hours, minutes, seconds);
        }
        else if minutes < 10 {
            hhmmss = format!("{}:0{}:{}", hours, minutes, seconds);
        }
        else if seconds < 10 {
            hhmmss = format!("{}:{}:0{}", hours, minutes, seconds);
        }
        else {
            hhmmss = format!("{}:{}:{}", hours, minutes, seconds);
        }
    }
    else {
        let minutes = u_duration / 60;
        let seconds = u_duration % 60;
        if seconds < 10 {
            hhmmss = format!("{}:0{}", minutes, seconds);
        }
        else {
            hhmmss = format!("{}:{}", minutes, seconds);
        }
    }
    
    hhmmss
}

pub fn ellipse_line(line: &str, max_width: usize) -> String {
    return if line.is_empty() { String::new() } 
    else if line.graphemes(true).count() > max_width {
        let clipped_line: String = line.graphemes(true).take(max_width - 4).collect();
        clipped_line + "..."
    } else { String::from(line) }
}