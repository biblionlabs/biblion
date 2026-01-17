use std::sync::Arc;

use freya::prelude::*;
use setup_core::{event, TantivySink};

use crate::utils::data_dir;

#[derive(Clone, PartialEq, PartialOrd)]
struct Bible {
    id: String,
    name: String,
    english_name: String,
    installed: bool,
    installing: bool,
    progress: f32,
}

#[derive(Clone, PartialEq, PartialOrd)]
struct Verse {
    bible: Bible,
    book: String,
    chapter: i32,
    verse: (i32, i32),
    text: String,
}

pub fn init() -> impl IntoElement {
    let mut theme = use_init_root_theme(|| PreferredTheme::Dark.to_theme());
    let mut search_state = use_state(String::new);
    let mut filtered_verses = use_state(Vec::new);

    use_side_effect(move || theme.set(PreferredTheme::Dark.to_theme()));

    let cache_dir = data_dir(&["cache"]);
    let database = Arc::new(TantivySink::from(data_dir(&["index"])));
    let source_variants = Arc::new(
        setup_core::SetupBuilder::new().cache_path(cache_dir)
        // Add Reina Valera 1960 Bible
        .add_bible_from_url(
            "spa_rv1960",
            "https://raw.githubusercontent.com/biblionlabs/extra_data_source/refs/heads/main/bibles/spa_rv1960/manifest.json", 
            "https://raw.githubusercontent.com/biblionlabs/extra_data_source/refs/heads/main/bibles/spa_rv1960/desc.json",
            Some("https://raw.githubusercontent.com/biblionlabs/extra_data_source/refs/heads/main/bibles/{bible_id}/books/{book}.json")
        )
        .on::<event::Error>({
            move |_e| {
                // Notification::new().summary("Worship Screens Failed to install Bible").body(&e).show().inspect_err(|e| error!("{e}")).unwrap();
            }})
        .on::<event::Progress>({
            move |(step_id, _current, _total)| {
                if step_id == "crossrefs" {
                    return;
                }
                // if let Some(window) = main_window.upgrade() {
                //     let state = window.global::<MainState>();
                //     let bibles = state.get_bibles();
                //     if let Some((idx, mut bible)) = bibles.iter().position(|b| b.id == step_id).and_then(|row| bibles.row_data(row).map(|b| (row, b))) {
                //         bible.installing = current != total;
                //         bible.installed = current == total;
                //         bible.progress = current as f32 / total as f32;
                //         if current == total {
                //             _ = Notification::new()
                //                 .summary("Worship Screens Bible Installed")
                //                 .body(&format!("{} success installed", bible.name.as_str()))
                //                 .show()
                //                 .inspect_err(|e| error!("{e}"));
                //         }
                //         bibles.set_row_data(idx, bible);
                //     }
                // }
                // if let Some(bibles_manager) = bibles_manager.get() {
                //     bibles_manager.update_progress(&step_id, current, total);
                // }
            }})
        .build().1
    );

    use_side_effect({
        let search_state = search_state.clone();
        let database = database.clone();
        move || {
            let s = search_state.read();
            let index = database.verse_index();
            let Ok(verses_found) =
                setup_core::service_db::SearchedVerse::from_search(s.as_str(), index)
            else {
                return;
            };
            let v = verses_found
                .iter()
                .map(|v| Verse {
                    bible: Bible {
                        english_name: v.bible.english_name.clone(),
                        id: v.bible.id.clone(),
                        installed: false,
                        installing: false,
                        name: v.bible.name.clone(),
                        progress: 0.0,
                    },
                    book: v.book.clone(),
                    chapter: v.chapter,
                    text: v.text.clone(),
                    verse: v.verse,
                })
                .collect::<Vec<_>>();
            filtered_verses.set(v);
        }
    });

    std::thread::spawn({
        let source_variants = source_variants.clone();
        let database = database.clone();
        move || {
            source_variants.install_cross(database.as_ref()).unwrap();
            source_variants
                .install_langs(database.as_ref(), &[])
                .unwrap();
        }
    });

    let filtered_verses = filtered_verses.read().clone();

    rect()
        .content(Content::Flex)
        .center()
        .padding(10.)
        .spacing(10.)
        .expanded()
        .vertical()
        .theme_background()
        .child(
            rect()
                .content(Content::Flex)
                .center()
                .width(Size::Inner)
                .padding(5.)
                .spacing(10.)
                .horizontal()
                .child(
                    Button::new()
                        .compact()
                        .padding(5.)
                        .child(label().text("Install Bible")),
                )
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
                .children(filtered_verses.iter().enumerate().map(|(i, verse)| {
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
                                            verse.book, verse.chapter, verse.verse.0
                                        ))
                                        .into_element(),
                                    label()
                                        .color(Color::WHITE)
                                        .text(verse.text.clone())
                                        .into_element(),
                                ]),
                        )
                        .into()
                })),
        )
}
