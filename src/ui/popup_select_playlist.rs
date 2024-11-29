use crate::app::{App, AppResult};
use crate::ui::utils;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(60, 60, frame.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);
    
    let new_playlist_entry = if app.app_flags.is_introducing_new_playlist_name { 
        format!("New Playlist (type new name) => {} ", app.new_name)
    } else {
        "New Playlist".to_string()
    };

    let mut items = vec![ListItem::from(Text::from(new_playlist_entry))];
    for playlist_id in app.database.alphabetical_playlists() {
        items.push(ListItem::from(Text::from(app.database.get_playlist(playlist_id).name())));
    }

    if app.list_states.popup_select_playlist_list_state.selected().is_none() {
        app.list_states.popup_select_playlist_list_state.select_first()
    }
    let popup_list = List::new(items)
        .style(Style::default().fg(Color::default()))
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);
    let popup_footer = Paragraph::new(Line::from("(CR) select playlist")).block(Block::default());

    let block = Block::bordered()
        .title(Line::raw("Add to playlist").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let inner = block.inner(chunks[0]);

    frame.render_widget(Clear, area);
    frame.render_widget(block, chunks[0]);
    frame.render_stateful_widget(popup_list, inner, &mut app.list_states.popup_select_playlist_list_state);
    frame.render_widget(popup_footer, chunks[1]);
    Ok(())
}
