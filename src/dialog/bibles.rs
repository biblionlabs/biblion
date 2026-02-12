use std::sync::Arc;
use std::thread;
use std::time::Duration;

use async_io::Timer;
use freya::prelude::*;
use futures::StreamExt;
use kanal::{Receiver, Sender, unbounded};
use setup_core::{Selection, TantivySink, event};

use crate::dialog::Dialog;
use crate::utils::data_dir;

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct BibleItem {
    pub id: String,
    pub name: String,
    pub english_name: String,
    pub installed: bool,
    pub installing: bool,
    pub progress: f32, // 0.0 .. 1.0
}

pub fn manage_bibles(mut show_dialog: State<bool>, database: Arc<TantivySink>) -> impl IntoElement {
    let mut search = use_state(String::new);
    let mut all_bibles = use_state(Vec::<BibleItem>::new);
    let mut filtered = use_state(Vec::<BibleItem>::new);

    let cache_dir = data_dir(&["cache"]);
    let (tx, rx): (Sender<(String, u64, u64)>, Receiver<(String, u64, u64)>) = unbounded();

    let setup = Arc::new(
        setup_core::SetupBuilder::new().cache_path(cache_dir)
        // Add Reina Valera 1960 Bible
        .add_bible_from_url(
            "spa_rv1960",
            "https://raw.githubusercontent.com/biblionlabs/extra_data_source/refs/heads/main/bibles/spa_rv1960/manifest.json", 
            "https://raw.githubusercontent.com/biblionlabs/extra_data_source/refs/heads/main/bibles/spa_rv1960/desc.json",
            Some("https://raw.githubusercontent.com/biblionlabs/extra_data_source/refs/heads/main/bibles/{bible_id}/books/{book}.json")
        )
        .on::<event::Message>(move |msg| println!("{msg}"))
        .on::<event::Error>({
            move |e| {
                println!("Fail to instal: {e}");
                // manejar error si se desea
            }})
        .on::<event::Progress>({
            let tx = tx.clone();
            move |(step_id, current, total)| {
                println!("Process: {step_id} ({current}/{total})");
                if step_id == "crossrefs" {
                    if current == total {
                        tracing::debug!("crossrefs finish");
                    }
                    return;
                }
                let _ = tx.send((step_id.clone(), current, total));
            }
        })
        .build().1
    );

    use_hook(|| {
        let setup = setup.clone();
        let database = database.clone();

        let list_res: Result<
            Vec<(
                String,
                String,
                String,
                String,
                setup_core::BibleInstallStatus,
            )>,
            _,
        > = setup.list_bibles(database.as_ref());

        if let Ok(list) = list_res {
            let items = list
                .into_iter()
                .map(|(id, name, english, _lang, status)| BibleItem {
                    id,
                    name,
                    english_name: english,
                    installed: status.is_complete(),
                    installing: false,
                    progress: (status.completion_percentage() as f32) / 100.0,
                })
                .collect::<Vec<_>>();
            all_bibles.set(items);
        }
    });

    use_hook(|| {
        let rx = rx.clone();

        spawn(async move {
            let mut interval = Timer::interval(Duration::from_millis(120));
            loop {
                interval.next().await;

                while let Ok(Some((step_id, current, total))) = rx.try_recv() {
                    all_bibles.with_mut(|mut bibles| {
                        if let Some(bible) = bibles.iter_mut().find(|b| b.id == step_id) {
                            let is_complete = current == total;
                            bible.installing = !is_complete;
                            bible.installed = is_complete;
                            bible.progress = if total > 0 {
                                current as f32 / total as f32
                            } else {
                                0.0
                            };
                        }
                    });
                }
            }
        });
    });

    use_side_effect({
        move || {
            let q = search.read().to_lowercase();
            let list = all_bibles
                .read()
                .iter()
                .filter(|b| {
                    q.is_empty()
                        || b.name.to_lowercase().contains(&q)
                        || b.english_name.to_lowercase().contains(&q)
                })
                .cloned()
                .collect::<Vec<_>>();
            filtered.set(list);
        }
    });

    let install_action = {
        let setup = setup.clone();
        let database = database.clone();
        move |bible_id: String| {
            {
                let mut current = all_bibles.read().clone();
                if let Some(b) = current.iter_mut().find(|x| x.id == bible_id) {
                    b.installing = true;
                    b.progress = 0.0;
                }
                all_bibles.set(current);
            }
            thread::spawn({
                let setup = setup.clone();
                let database = database.clone();
                let bible_id = bible_id.clone();
                move || {
                    if let Err(e) = setup.run_with_sink(
                        Selection {
                            bibles: vec![bible_id.clone()],
                            ..Default::default()
                        },
                        database.as_ref(),
                    ) {
                        tracing::error!("Error instalando biblia {bible_id}: {e}");
                    }
                }
            });
        }
    };

    let filtered = filtered.read().clone();
    let filtered_len = filtered.len();

    if !*show_dialog.read() {
        return rect().into_element();
    }

    Dialog::new("Manage Bibles".to_string())
        .width(Size::px(640.))
        .on_close_request({
            move |()| {
                show_dialog.set(false);
            }
        })
        .child(
            rect()
                .vertical()
                .spacing(10.)
                .padding(8.)
                .max_height(Size::window_percent(60.))
                .children([
                    label()
                        .text("Download and install Bible translations")
                        .color(Color::from_hex("#cfcfcf").unwrap())
                        .font_size(14.)
                        .into(),
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .child(
                            Input::new()
                                .width(Size::Fill)
                                .placeholder("Search: Reina Valera 1960, KJV...")
                                .value(search.read().clone())
                                .on_change(move |v| search.set(v)),
                        )
                        .into_element(),
                    VirtualScrollView::new_with_data(filtered, move |i, filtered| {
                        let b = &filtered[i];
                        rect()
                            .key(i)
                            .rounded()
                            .expanded()
                            .max_height(Size::px(60.))
                            .background(Color::from_hex("#2C2C2C").unwrap())
                            .content(Content::Flex)
                            .vertical()
                            .margin(Gaps::new(10., 0., 0., 0.))
                            .children([
                                rect()
                                    .spacing(6.)
                                    .padding(8.)
                                    .horizontal()
                                    .width(Size::Fill)
                                    .main_align(Alignment::SpaceBetween)
                                    .cross_align(Alignment::Center)
                                    .children([
                                        rect()
                                            .max_width(Size::px(300.))
                                            .vertical()
                                            .children([
                                                label()
                                                    .text({
                                                        if !b.english_name.is_empty() {
                                                            b.english_name.clone()
                                                        } else {
                                                            b.name.clone()
                                                        }
                                                    })
                                                    .max_lines(1)
                                                    .text_overflow(TextOverflow::Ellipsis)
                                                    .font_weight(FontWeight::BOLD)
                                                    .color(Color::WHITE)
                                                    .into_element(),
                                                label()
                                                    .text({
                                                        if !b.name.is_empty()
                                                            && b.name != b.english_name
                                                        {
                                                            b.name.clone()
                                                        } else {
                                                            "".to_string()
                                                        }
                                                    })
                                                    .max_lines(1)
                                                    .text_overflow(TextOverflow::Ellipsis)
                                                    .color(Color::from_hex("#bdbdbd").unwrap())
                                                    .font_size(13.)
                                                    .into_element(),
                                            ])
                                            .into_element(),
                                        rect()
                                            .horizontal()
                                            .spacing(8.)
                                            .child(if b.installed {
                                                label()
                                                    .text("Installed")
                                                    .color(Color::from_hex("#27ae60").unwrap())
                                                    .font_weight(FontWeight::BOLD)
                                                    .into_element()
                                            } else if b.installing {
                                                label()
                                                    .text("Installing...")
                                                    .color(Color::from_hex("#f39c12").unwrap())
                                                    .into_element()
                                            } else {
                                                Button::new()
                                                    .compact()
                                                    .on_press({
                                                        let id = b.id.clone();
                                                        let mut install_action =
                                                            install_action.clone();
                                                        move |_| install_action(id.clone())
                                                    })
                                                    .child(label().text("Install").into_element())
                                                    .into_element()
                                            })
                                            .into_element(),
                                    ])
                                    .into_element(),
                                ProgressBar::new((b.progress * 100.0).max(0.0).min(100.0))
                                    .height(5.)
                                    .width(Size::Fill)
                                    .into_element(),
                            ])
                            .into_element()
                    })
                    .item_size(75.)
                    .length(filtered_len as i32)
                    .expanded()
                    .direction(Direction::Vertical)
                    .into_element(),
                ]),
        )
        .action(
            Button::new()
                .expanded()
                .filled()
                .on_press(move |_| show_dialog.set(false))
                .child(label().text("Done")),
        )
        .into_element()
}
