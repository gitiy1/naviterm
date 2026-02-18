use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, Padding, Paragraph};
use ratatui::Frame;
use crate::app::{App, AppResult};
use crate::ui::utils;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(35, 80, frame.area());

    // Build the keybindings list
    let items = build_keybindings_list(app);
    
    // Ensure list state is within bounds
    if items.is_empty() {
        app.list_states.popup_keybindings_list_state.select(None);
    } else {
        let selected = app.list_states.popup_keybindings_list_state.selected();
        if selected.is_none() || selected.unwrap() >= items.len() {
            app.list_states.popup_keybindings_list_state.select_first();
        }
    }
    
    // Create list items
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| {
            if item.0.is_empty() {
                // Section header
                ListItem::new(Line::from(vec![
                    Span::styled(
                        item.1.clone(),
                        Style::default()
                            .fg(app.app_colors.primary_accent)
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                    ),
                ]))
            } else {
                // Keybinding entry
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<15}", item.0),
                        Style::default()
                            .fg(app.app_colors.primary_accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::raw(item.1.clone()),
                ]))
            }
        })
        .collect();

    let title = if app.app_flags.is_searching_keybindings {
        format!("Keybindings (search: {})", app.search_data.keybindings_search_string)
    } else {
        "Keybindings".to_string()
    };

    let list = List::new(list_items)
        .block(
            Block::bordered()
                .title(title)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .padding(Padding::new(2, 2, 1, 1)),
        )
        .style(Style::default().fg(Color::default()).bg(Color::default()))
        .highlight_style(
            Style::default()
                .fg(app.app_colors.primary_accent)
                .add_modifier(Modifier::BOLD),
        );

    // Footer with help text
    let footer_text = if app.app_flags.is_searching_keybindings {
        "Enter: exit search  Esc: clear  Type to filter"
    } else {
        "/: search  j/k: scroll  Ctrl-d/u: page down/up  q/Esc: close"
    };
    
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            footer_text,
            Style::default().fg(app.app_colors.secondary_accent),
        ),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(Clear, area);
    frame.render_stateful_widget(
        list,
        area,
        &mut app.list_states.popup_keybindings_list_state,
    );
    
    // Render footer at bottom of popup area
    let footer_area = ratatui::layout::Rect {
        x: area.x,
        y: area.y + area.height - 2,
        width: area.width,
        height: 1,
    };
    frame.render_widget(footer, footer_area);

    Ok(())
}

fn build_keybindings_list(app: &App) -> Vec<(String, String)> {
    let screen = app.current_screen.as_str();
    let groups = app.shortcuts.get_all_keybindings_for_screen(screen);
    
    let mut items: Vec<(String, String)> = Vec::new();
    
    for group in groups {
        // Add section header
        items.push((String::new(), group.section_name));
        
        // Add bindings
        for (key, desc) in group.bindings {
            items.push((key, desc));
        }
    }
    
    // Apply search filter if active
    if !app.search_data.keybindings_search_string.is_empty() {
        let search_lower = app.search_data.keybindings_search_string.to_lowercase();
        items = items
            .into_iter()
            .filter(|(key, desc)| {
                // Keep section headers and empty lines
                if key.is_empty() && !desc.is_empty() {
                    return true; // Section header
                }
                if key.is_empty() && desc.is_empty() {
                    return false; // Skip empty lines during search
                }
                // Filter keybindings
                key.to_lowercase().contains(&search_lower)
                    || desc.to_lowercase().contains(&search_lower)
            })
            .collect();
    }
    
    items
}
