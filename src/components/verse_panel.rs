use std::sync::Arc;

use freya::prelude::*;
use setup_core::{DbSink, TantivySink};

pub struct VersePanel {
    database: Arc<TantivySink>,
    selected_verse: State<Option<(String, String, usize, usize)>>,
    on_word_click: Box<dyn Fn(String) + 'static>,
    layout: LayoutData,
}

impl PartialEq for VersePanel {
    fn eq(&self, other: &Self) -> bool {
        self.selected_verse == other.selected_verse && self.layout == other.layout
    }
}

impl VersePanel {
    pub fn new(database: Arc<TantivySink>) -> Self {
        Self {
            database,
            layout: LayoutData::default(),
            selected_verse: State::create(None),
            on_word_click: Box::new(|_| {}),
        }
    }

    pub fn selected_verse(
        mut self,
        selected_verse: impl Into<State<Option<(String, String, usize, usize)>>>,
    ) -> Self {
        self.selected_verse = selected_verse.into();
        self
    }

    pub fn on_word_click(mut self, on_click: impl Fn(String) + 'static) -> Self {
        self.on_word_click = Box::new(on_click);
        self
    }
}

impl LayoutExt for VersePanel {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}
impl ContainerExt for VersePanel {}
impl ContainerWithContentExt for VersePanel {}

impl Component for VersePanel {
    fn render(&self) -> impl IntoElement {
        let mut active_tab = use_state(|| 0);
        let mut selected_verse = self.selected_verse;
        let database = self.database.clone();

        let search = selected_verse.read();
        let Some(search) = search.as_ref() else {
            return rect().into_element();
        };
        let (bible_id, book_id, chapter_idx, verse_idx) = search;
        let Ok(Some(chapter)) = database
            .get_crossreferences(&bible_id, &book_id, *chapter_idx as _, *verse_idx as _)
            .inspect_err(|e| println!("Failed to found crossref: {e}"))
        else {
            return rect().into_element();
        };

        // Construir los spans del párrafo con todos los versículos
        let mut paragraph_spans = Vec::new();
        let mut char_count = 0;
        let mut highlight_start = 0;
        let mut highlight_end = 0;

        for (idx, v) in chapter.verses.iter().enumerate() {
            let is_clicked = idx == v.verse_number as usize;
            let base_color = if is_clicked {
                Color::from_hex("#FFFFFF").unwrap()
            } else {
                Color::from_hex("#CCCCCC").unwrap()
            };

            let verse_number_text = format!(
                "{space}{} ",
                v.verse_number,
                space = if idx > 0 { " " } else { "" }
            );
            paragraph_spans.push(
                Span::new(verse_number_text.clone())
                    .color(Color::LIGHT_GRAY.with_a(75))
                    .font_weight(FontWeight::BOLD),
            );

            if v.highlighted {
                highlight_start = char_count;
            }
            char_count += verse_number_text.len();

            paragraph_spans.push(Span::new(v.text.clone()).color(base_color));

            char_count += v.text.len();

            if v.highlighted {
                highlight_end = char_count;
            }
        }

        let highlights = vec![(highlight_start + 1, highlight_end)];
        let cross_refs = chapter
            .verses
            .iter()
            .flat_map(|v| v.cross_references.clone())
            .collect::<Vec<_>>();
        let cross_refs_len = cross_refs.len();

        rect()
            .height(Size::Fill)
            .content(Content::Flex)
            .vertical()
            .background(Color::from_hex("#1E1E1E").unwrap())
            .padding(15.0)
            .spacing(10.0)
            .child(
                rect()
                    .content(Content::Flex)
                    .horizontal()
                    .width(Size::Fill)
                    .main_align(Alignment::SpaceBetween)
                    .padding(5.0)
                    .child(
                        label()
                            .color(Color::WHITE)
                            .font_size(18.0)
                            .font_weight(FontWeight::BOLD)
                            .text(format!("{} {}", chapter.book_name, chapter.chapter)),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| selected_verse.set(None))
                            .background(Color::from_hex("#2C2C2C").unwrap())
                            .hover_background(Color::from_hex("#353535").unwrap())
                            .padding(5.0)
                            .child(label().color(Color::WHITE).text("✕")),
                    ),
            )
            .child(
                ScrollView::new()
                    .height(Size::percent(50.0))
                    .direction(Direction::Vertical)
                    .child(
                        paragraph()
                            .width(Size::Fill)
                            .padding(10.0)
                            .spans_iter(paragraph_spans.into_iter())
                            .highlights(Some(highlights))
                            .highlight_color(Color::from_hex("#3A3A3A").unwrap()),
                    ),
            )
            .child(
                rect()
                    .height(Size::px(1.0))
                    .background(Color::from_hex("#2C2C2C").unwrap()),
            )
            .child(
                rect()
                    .max_height(Size::px(50.))
                    .content(Content::Flex)
                    .horizontal()
                    .padding(5.0)
                    .child(
                        Button::new()
                            .height(Size::Fill)
                            .on_press(move |_| active_tab.set(0))
                            .corner_radius(CornerRadius {
                                top_left: 8.,
                                ..Default::default()
                            })
                            .background(if *active_tab.read() == 0 {
                                Color::from_hex("#3A3A3A").unwrap()
                            } else {
                                Color::from_hex("#2C2C2C").unwrap()
                            })
                            .hover_background(Color::from_hex("#353535").unwrap())
                            .padding(8.0)
                            .child(
                                rect()
                                    .center()
                                    .horizontal()
                                    .spacing(5.)
                                    .child(
                                        rect()
                                            .center()
                                            .text_align(TextAlign::Center)
                                            .rounded_full()
                                            .padding(8.)
                                            .background(Color::DARK_GRAY)
                                            .child(
                                                label()
                                                    .color(Color::WHITE)
                                                    .text(cross_refs_len.to_string()),
                                            ),
                                    )
                                    .child(
                                        label()
                                            .color(Color::WHITE)
                                            .text_align(TextAlign::Center)
                                            .text("Cross References"),
                                    ),
                            ),
                    )
                    .child(
                        Button::new()
                            .height(Size::Fill)
                            .on_press(move |_| active_tab.set(1))
                            .corner_radius(CornerRadius {
                                top_right: 8.,
                                ..Default::default()
                            })
                            .background(if *active_tab.read() == 1 {
                                Color::from_hex("#3A3A3A").unwrap()
                            } else {
                                Color::from_hex("#2C2C2C").unwrap()
                            })
                            .hover_background(Color::from_hex("#353535").unwrap())
                            .padding(8.0)
                            .child(label().color(Color::WHITE).text("Glossary")),
                    ),
            )
            .child(
                rect()
                    .height(Size::Fill)
                    .padding(10.0)
                    .content(Content::Flex)
                    .child(if *active_tab.read() == 0 {
                        VirtualScrollView::new_with_data(cross_refs, move |i, cross_refs| {
                            let cross_ref = &cross_refs[i];
                            rect()
                                .key(i)
                                .padding((0., 0., 10., 0.))
                                .child(
                                    rect()
                                        .background(Color::from_hex("#2C2C2C").unwrap())
                                        .rounded()
                                        .padding(10.0)
                                        .content(Content::Flex)
                                        .vertical()
                                        .spacing(5.0)
                                        .width(Size::Fill)
                                        .child(
                                            label()
                                                .color(Color::from_hex("#888888").unwrap())
                                                .font_size(12.0)
                                                .font_weight(FontWeight::BOLD)
                                                .text(format!(
                                                    "{} {}:{}",
                                                    cross_ref.book_name,
                                                    cross_ref.chapter,
                                                    cross_ref.verse
                                                )),
                                        )
                                        .child(
                                            label()
                                                .color(Color::from_hex("#CCCCCC").unwrap())
                                                .font_size(14.0)
                                                .text(cross_ref.text.clone()),
                                        ),
                                )
                                .into_element()
                        })
                        .length(cross_refs_len as i32)
                        .item_size(80.)
                        .expanded()
                        .direction(Direction::Vertical)
                        .into_element()
                    } else {
                        rect()
                            .content(Content::Flex)
                            .center()
                            .main_align(Alignment::Center)
                            .cross_align(Alignment::Center)
                            .height(Size::Fill)
                            .child(
                                label()
                                    .color(Color::from_hex("#888888").unwrap())
                                    .font_size(16.0)
                                    .text("Under construction..."),
                            )
                            .into_element()
                    }),
            )
            .into_element()
    }
}
