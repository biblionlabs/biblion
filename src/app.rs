use std::sync::Arc;

use freya::prelude::*;
use setup_core::{DbSink, TantivySink};

use crate::components::{Toolbar, ToolbarItem};
use crate::dialog::manage_bibles;
use crate::utils::data_dir;

pub fn init() -> impl IntoElement {
    let mut theme = use_init_root_theme(|| PreferredTheme::Dark.to_theme());
    let mut show_bible_manager = use_state(|| false);
    let mut search_state = use_state(String::new);
    let mut filtered_verses = use_state(Vec::new);

    let database = Arc::new(TantivySink::from(data_dir(&["index"])));

    use_side_effect(move || theme.set(PreferredTheme::Dark.to_theme()));

    use_side_effect({
        let search_state = search_state.clone();
        let database = database.clone();
        move || {
            let s = search_state.read();
            let Ok(verses_found) = database.search_full_chapter(s.as_str(), None) else {
                return;
            };
            filtered_verses.set(verses_found);
        }
    });

    let filtered_verses = filtered_verses.read().clone();

    rect()
        .content(Content::Flex)
        .expanded()
        .vertical()
        .theme_background()
        .child(Toolbar::new().child(ToolbarItem::new(
            "Tools".to_string(),
            Menu::new().child(MenuButton::new().child("Install Bible").on_press(move |_| {
                show_bible_manager.set(true);
                ContextMenu::close();
            })),
        )))
        .child(manage_bibles(show_bible_manager, database))
        .child(
            rect()
                .content(Content::Flex)
                .padding(10.)
                .spacing(10.)
                .expanded()
                .vertical()
                .child(
                    rect()
                        .content(Content::Flex)
                        .center()
                        .width(Size::Inner)
                        .padding(5.)
                        .spacing(10.)
                        .horizontal()
                        .child(
                            Input::new()
                                .auto_focus(true)
                                .width(Size::Fill)
                                .placeholder("Search: Juan 1:3")
                                .value(search_state.read().clone())
                                .on_change(move |search| search_state.set(search)),
                        ),
                )
                .child(
                    ScrollView::new()
                        .expanded()
                        .direction(Direction::Vertical)
                        .scroll_with_arrows(true)
                        .spacing(10.)
                        .children(filtered_verses.iter().flat_map(|verse| {
                            verse
                                .verses
                                .iter()
                                .enumerate()
                                .flat_map(|(i, content)| {
                                    if !content.highlighted {
                                        return None;
                                    }
                                    Some(
                                        Button::new()
                                            .background(Color::from_hex("#2C2C2C").unwrap())
                                            .hover_background(Color::from_hex("#353535").unwrap())
                                            .child(
                                                rect()
                                                    .key(i)
                                                    .rounded()
                                                    .vertical()
                                                    .spacing(5.)
                                                    .padding(5.)
                                                    .width(Size::fill())
                                                    .content(Content::Flex)
                                                    .children([
                                                        label()
                                                            .color(Color::WHITE)
                                                            .font_weight(FontWeight::BOLD)
                                                            .text(format!(
                                                                "{} {}:{}",
                                                                verse.book_name,
                                                                verse.chapter,
                                                                content.verse_number
                                                            ))
                                                            .into_element(),
                                                        label()
                                                            .color(Color::WHITE)
                                                            .text(content.text.clone())
                                                            .into_element(),
                                                    ]),
                                            )
                                            .into_element(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })),
                ),
        )
}
