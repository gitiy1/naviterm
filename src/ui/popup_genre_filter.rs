use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph};
use ratatui::widgets::BorderType::Rounded;
use crate::app::{App, AppResult};
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {

    let area = utils::centered_rect(60, 60, frame.size());

    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Min(5),
        Constraint::Length(1),
    ]).split(area);

    let mut items = vec![ListItem::from(Text::from("Any"))];
    for genre in app.database.genres() {
        items.push(ListItem::from(Text::from(genre.clone())))
    }
    

    if app.popup_genre_list_state.selected().is_none() { app.popup_genre_list_state.select_first() }
    let popup_list = List::new(items).style(Style::default().fg(Color::default())).highlight_symbol("-> ").highlight_spacing(HighlightSpacing::Always);
    let popup_footer = Paragraph::new(Line::from("(CR) select genre")).block(Block::default());

    let block = Block::bordered()
        .title(Line::raw("Genre filter").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let inner = block.inner(chunks[0]);
    
    frame.render_widget(Clear, area);
    frame.render_widget(block, chunks[0]);
    frame.render_stateful_widget(popup_list, inner, &mut app.popup_genre_list_state);
    frame.render_widget(popup_footer, chunks[1]);
    Ok(())
}