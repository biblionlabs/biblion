use std::sync::Arc;

use freya::animation::*;
use freya::prelude::*;
use setup_core::TantivySink;

use crate::components::{Toolbar, ToolbarItem, VersePanel};
use crate::dialog::manage_bibles;
use crate::utils::data_dir;

pub fn init() -> impl IntoElement {
    let mut theme = use_init_root_theme(|| PreferredTheme::Dark.to_theme());
    let mut show_bible_manager = use_state(|| false);
    let mut search_state = use_state(String::new);
    let mut filtered_verses = use_state(Vec::new);
    let mut selected_verse = use_state(|| None::<(String, String, usize, usize)>);

    let database = Arc::new(TantivySink::from(data_dir(&["index"])));
    let platform = Platform::get();
    let root_size = platform.root_size.read().width;
    let root_size = if root_size < 768.0 {
        100.
    } else if root_size < 1024.0 {
        50.
    } else {
        35.
    };

    let mut panel_width_anim =
        use_animation_with_dependencies(&root_size, |c, root_size| {
            c.on_change(OnChange::Finish);
            AnimNum::new(0., *root_size)
                .function(Function::Sine)
                .ease(Ease::InOut)
                .time(300)
        });

    let mut is_panel_open = use_state(|| false);

    use_side_effect(move || theme.set(PreferredTheme::Dark.to_theme()));

    use_side_effect({
        let search_state = search_state.clone();
        let database = database.clone();
        move || {
            let s = search_state.read();
            let index = database.verse_index();
            let Ok(verses_found) =
                setup_core::service_db::SearchedVerse::from_search(s.as_str(), index, Some(50))
            else {
                return;
            };
            filtered_verses.set(verses_found);
        }
    });

    let filtered_verses_data = filtered_verses.read().clone();
    let should_show_panel = selected_verse.read().is_some();

    if should_show_panel != *is_panel_open.read() {
        if should_show_panel {
            panel_width_anim.start();
        } else {
            panel_width_anim.reverse();
        }
        is_panel_open.set(should_show_panel);
    }

    let panel_width_value = panel_width_anim.read().value();

    let search_panel_percentage = 100.0 - panel_width_value;

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
        .child(manage_bibles(show_bible_manager, database.clone()))
        .child(
            rect()
                .content(Content::Flex)
                .padding(10.)
                .spacing(10.)
                .expanded()
                .horizontal()
                .maybe(panel_width_value < 80., |r| {
                    r.child(
                        rect()
                            .width(Size::percent(search_panel_percentage))
                            .content(Content::Flex)
                            .spacing(10.)
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
                                    .children(filtered_verses_data.iter().map(|verse| {
                                        let bible_id = verse.bible.id.clone();
                                        let book_id = verse.book_id.clone();
                                        let ch_idx = verse.chapter;
                                        let v_idx = verse.verse.0;
                                        Button::new()
                                            .background(Color::from_hex("#2C2C2C").unwrap())
                                            .hover_background(Color::from_hex("#353535").unwrap())
                                            .on_press({
                                                move |_| {
                                                    selected_verse.set(Some((
                                                        bible_id.clone(),
                                                        book_id.clone(),
                                                        ch_idx as _,
                                                        v_idx as _,
                                                    )));
                                                }
                                            })
                                            .child(
                                                rect()
                                                    .key(v_idx)
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
                                                                verse.book, verse.chapter, v_idx
                                                            ))
                                                            .into_element(),
                                                        label()
                                                            .color(Color::WHITE)
                                                            .text(verse.text.clone())
                                                            .into_element(),
                                                    ]),
                                            )
                                            .into_element()
                                    })),
                            ),
                    )
                })
                .maybe(panel_width_value > 0.001, {
                    let database = database.clone();
                    |r| {
                        r.child(
                            VersePanel::new(database)
                                .width(Size::percent(panel_width_value))
                                .selected_verse(selected_verse)
                                .on_word_click(|word| {
                                    println!("Word clicked: {}", word);
                                })
                                .into_element(),
                        )
                    }
                }),
        )
}
