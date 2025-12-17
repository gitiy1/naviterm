use crate::app::{App, AppResult};
use crate::mappings::ShortcutAction;
use crate::ui::utils;
use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::Layout;
use ratatui::prelude::{Modifier, Span};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph};
use ratatui::Frame;

pub fn draw_popup(app: &mut App, frame: &mut Frame) -> AppResult<()> {
    let area = utils::centered_rect(60, 60, frame.area());

    let block = Block::bordered()
        .title(Line::raw("Genre filter").centered())
        .padding(Padding::new(4, 4, 1, 1))
        .border_type(Rounded);

    let mut items = vec![ListItem::from(Text::from("Any"))];
    for genre in app.database.genres() {
        items.push(ListItem::from(Text::from(genre.clone())))
    }

    if app.list_states.popup_genre_list_state.selected().is_none() {
        app.list_states.popup_genre_list_state.select_first()
    }

    let popup_list = List::new(items)
        .style(Style::default().fg(Color::default()))
        .highlight_symbol("-> ")
        .highlight_spacing(HighlightSpacing::Always);

    let favorites_header = Paragraph::new(
        Line::from("Favorites").style(
            Style::default()
                .fg(app.app_colors.primary_accent)
                .add_modifier(Modifier::BOLD),
        ),
    );

    let mut favorites_items: Vec<ListItem> = vec![];
    if app.database.favorite_genres().is_empty() {
        favorites_items.push(
            ListItem::from(Line::from("No favorite genres")).style(
                Style::default()
                    .fg(app.app_colors.secondary_accent)
                    .add_modifier(Modifier::ITALIC),
            ),
        );
    } else {
        for (index, genre) in app.database.favorite_genres().iter().enumerate() {
            favorites_items.push(ListItem::from(Line::from(vec![
                Span {
                    content: "(".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: (index + 1).to_string().into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: ") ".into(),
                    style: Style::default()
                        .fg(app.app_colors.primary_accent)
                        .add_modifier(Modifier::BOLD),
                },
                Span {
                    content: genre.into(),
                    style: Style::default(),
                },
            ])));
        }
    }

    let favorites = List::new(favorites_items);

    let popup_footer = Paragraph::new(vec![
        Line::from(""),
        Line::from(format!(
            "{} select genre, {} toggle favorite, (1-9) select favorite",
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupGenreAcceptSelected, None),
            app.shortcuts
                .get_key_combo_for_operation(ShortcutAction::PopupGenreToggleFavorite, None)
        ))
        .style(
            Style::default()
                .fg(app.app_colors.secondary_accent)
                .add_modifier(Modifier::ITALIC),
        ),
    ]);

    let inner = block.inner(area);
    let popup_layout = Layout::vertical([Min(5), Length(2)]);
    let popup_sections = Layout::horizontal([Percentage(50), Percentage(50)]);
    let favorites_layout = Layout::vertical([Length(2), Min(5)]);

    let [popup_area, footer_area] = popup_layout.areas(inner);
    let [genres_area, favorites_area] = popup_sections.areas(popup_area);
    let [favorites_header_area, favorites_list_area] = favorites_layout.areas(favorites_area);

    frame.render_widget(Clear, area);
    frame.render_stateful_widget(
        popup_list,
        genres_area,
        &mut app.list_states.popup_genre_list_state,
    );
    frame.render_widget(popup_footer, footer_area);
    frame.render_widget(favorites_header, favorites_header_area);
    frame.render_widget(favorites, favorites_list_area);
    frame.render_widget(block, area);
    Ok(())
}
