use crate::app::{App, AppResult};
use crate::ui::utils::{get_text_for_song_item_queue};
use ratatui::layout::Rect;
use ratatui::text::{Line};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw_tab(app: &mut App, area: Rect, frame: &mut Frame) -> AppResult<()> {
    let block = Block::bordered().border_type(Rounded);

    if app.queue.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from("Nothing in the queue..."))
                .block(Block::default())
                .block(block),
            area,
        );
    } else {
        let mut items: Vec<ListItem> = Vec::new();
        for (index, song_id) in app.queue.iter().enumerate() {
            items.push(get_text_for_song_item_queue(
                &app.database,
                &app.app_flags,
                app.list_states.queue_list_state.selected().unwrap(),
                index,
                song_id,
                &app.search_data,
                &app.queue_order,
                app.index_in_queue
            ));
        }
        let list = List::new(items)
            .block(block)
            .highlight_symbol("-> ")
            .highlight_spacing(HighlightSpacing::Always);
        if app.app_flags.move_to_next_in_search {
            app.app_flags.move_to_next_in_search = false;
            app.list_states.queue_list_state.select(Some(
                *app.search_data.search_results_indexes.get(app.search_data.index_in_search).unwrap(),
            ));
        }
        frame.render_stateful_widget(list, area, &mut app.list_states.queue_list_state);
    }

    Ok(())
}
