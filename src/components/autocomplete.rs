use std::borrow::Cow;

use freya::text_edit::{EditableConfig, EditableEvent, EditorLine, TextEditor, use_editable};
use freya::{animation::*, prelude::*};

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum AutoCompleteInputStatus {
    #[default]
    Idle,
    Hovering,
}

#[derive(Clone, PartialEq)]
pub struct AutoCompleteInput {
    pub(crate) theme_colors: Option<InputColorsThemePartial>,
    pub(crate) theme_layout: Option<InputLayoutThemePartial>,
    value: State<String>,
    suggestions: Vec<String>,
    placeholder: Option<Cow<'static, str>>,
    on_submit: Option<EventHandler<String>>,
    auto_focus: bool,
    width: Size,
    enabled: bool,
    key: DiffKey,
    text_align: TextAlign,
    a11y_id: Option<AccessibilityId>,
}

impl KeyExt for AutoCompleteInput {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl AutoCompleteInput {
    pub fn new(value: impl Into<State<String>>, suggestions: impl Into<Vec<String>>) -> Self {
        AutoCompleteInput {
            theme_colors: None,
            theme_layout: None,
            value: value.into(),
            suggestions: suggestions.into(),
            placeholder: None,
            on_submit: None,
            auto_focus: false,
            width: Size::px(200.),
            enabled: true,
            key: DiffKey::default(),
            text_align: TextAlign::default(),
            a11y_id: None,
        }
    }

    pub fn enabled(mut self, enabled: impl Into<bool>) -> Self {
        self.enabled = enabled.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'static, str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn on_submit(mut self, on_submit: impl Into<EventHandler<String>>) -> Self {
        self.on_submit = Some(on_submit.into());
        self
    }

    pub fn auto_focus(mut self, auto_focus: impl Into<bool>) -> Self {
        self.auto_focus = auto_focus.into();
        self
    }

    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = width.into();
        self
    }

    pub fn theme_colors(mut self, theme: InputColorsThemePartial) -> Self {
        self.theme_colors = Some(theme);
        self
    }

    pub fn theme_layout(mut self, theme: InputLayoutThemePartial) -> Self {
        self.theme_layout = Some(theme);
        self
    }

    pub fn text_align(mut self, text_align: impl Into<TextAlign>) -> Self {
        self.text_align = text_align.into();
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }

    pub fn a11y_id(mut self, a11y_id: impl Into<AccessibilityId>) -> Self {
        self.a11y_id = Some(a11y_id.into());
        self
    }
}

fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let clean_text = simplify(text);
    let clean_pattern = simplify(pattern);
    clean_text.contains(&clean_pattern) && text != pattern
}

fn simplify(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            _ => c,
        })
        .collect()
}

impl Component for AutoCompleteInput {
    fn render(&self) -> impl IntoElement {
        let focus = use_hook(|| Focus::new_for_id(self.a11y_id.unwrap_or_else(Focus::new_id)));
        let focus_status = use_focus_status(focus);
        let holder = use_state(ParagraphHolder::default);
        let mut area = use_state(Area::default);
        let mut status = use_state(AutoCompleteInputStatus::default);
        let mut editable = use_editable(|| self.value.read().to_string(), EditableConfig::new);
        let mut is_dragging = use_state(|| false);
        let mut ime_preedit = use_state(|| None);
        let mut value = self.value.clone();
        let mut selected_index = use_state(|| -1i32);
        let mut open = use_state(|| false);
        let mut user_text = use_state(String::new);

        let theme_colors = get_theme!(&self.theme_colors, input);
        let theme_layout = get_theme!(&self.theme_layout, input_layout);

        let enabled = use_reactive(&self.enabled);
        use_drop(move || {
            if status() == AutoCompleteInputStatus::Hovering && enabled() {
                Cursor::set(CursorIcon::default());
            }
        });

        let display_placeholder = value.read().is_empty() && self.placeholder.is_some();
        let on_submit = self.on_submit.clone();

        if &*value.read() != editable.editor().read().rope() {
            editable.editor_mut().write().set(&value.read());
            editable.editor_mut().write().editor_history().clear();
        }

        // Filter suggestions
        let current_value = value.read().clone();
        let filtered_suggestions: Vec<String> = if current_value.is_empty() {
            open.set(false);
            vec![]
        } else {
            if !open() {
                open.set(true);
            }
            self.suggestions
                .iter()
                .filter(|s| fuzzy_match(s, &current_value))
                .cloned()
                .collect()
        };

        // Get ghost text
        let ghost_text = if !current_value.is_empty() && !filtered_suggestions.is_empty() {
            let best_match = if selected_index() >= 0
                && (selected_index() as usize) < filtered_suggestions.len()
            {
                &filtered_suggestions[selected_index() as usize]
            } else {
                &filtered_suggestions[0]
            };

            if best_match
                .to_lowercase()
                .starts_with(&current_value.to_lowercase())
            {
                Some(best_match[current_value.len()..].to_string())
            } else {
                None
            }
        } else {
            None
        };

        let mut animation = use_animation(move |conf| {
            conf.on_creation(OnCreation::Run);

            let scale = AnimNum::new(0.95, 1.)
                .time(150)
                .ease(Ease::Out)
                .function(Function::Expo);
            let opacity = AnimNum::new(0., 1.)
                .time(150)
                .ease(Ease::Out)
                .function(Function::Expo);
            (scale, opacity)
        });

        use_side_effect(move || {
            if open() {
                animation.start();
            } else {
                animation.reverse();
            }
        });

        let on_ime_preedit = move |e: Event<ImePreeditEventData>| {
            ime_preedit.set(Some(e.data().text.clone()));
        };

        let suggestions_clone = filtered_suggestions.clone();
        let on_key_down = move |e: Event<KeyboardEventData>| match &e.key {
            Key::Named(NamedKey::Enter) => {
                if selected_index() >= 0 && (selected_index() as usize) < suggestions_clone.len() {
                    let selected = suggestions_clone[selected_index() as usize].clone();
                    *value.write() = selected.clone();
                    let mut editor = editable.editor_mut().write();
                    editor.set(&selected);
                    editor.move_cursor_to(selected.len());
                    open.set(false);
                    selected_index.set(-1);
                    if let Some(on_submit) = &on_submit {
                        on_submit.call(selected);
                    }
                } else if let Some(on_submit) = &on_submit {
                    let text = editable.editor().peek().to_string();
                    on_submit.call(text);
                    open.set(false);
                }
            }
            Key::Named(NamedKey::ArrowDown) => {
                e.stop_propagation();
                if !suggestions_clone.is_empty() {
                    if !open() {
                        open.set(true);
                    }
                    let new_index = selected_index() + 1;
                    if new_index >= suggestions_clone.len() as i32 {
                        selected_index.set(-1);
                        let user_text_val = user_text.read().clone();
                        *value.write() = user_text_val.clone();
                        let mut editor = editable.editor_mut().write();
                        editor.set(&user_text_val);
                        editor.move_cursor_to(user_text_val.len());
                    } else {
                        selected_index.set(new_index);
                    }
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                e.stop_propagation();
                if !suggestions_clone.is_empty() {
                    if !open() {
                        open.set(true);
                    }
                    let new_index = selected_index() - 1;
                    if new_index < -1 {
                        selected_index.set(suggestions_clone.len() as i32 - 1);
                    } else if new_index == -1 {
                        let user_text_val = user_text.read().clone();
                        *value.write() = user_text_val.clone();
                        let mut editor = editable.editor_mut().write();
                        editor.set(&user_text_val);
                        editor.move_cursor_to(user_text_val.len());
                        selected_index.set(-1);
                    } else {
                        selected_index.set(new_index);
                    }
                }
            }
            Key::Named(NamedKey::Escape) => {
                open.set(false);
                focus.request_unfocus();
                selected_index.set(-1);
            }
            key if *key != Key::Named(NamedKey::Enter) && *key != Key::Named(NamedKey::Meta) => {
                e.stop_propagation();
                editable.process_event(EditableEvent::KeyDown {
                    key: &e.key,
                    modifiers: e.modifiers,
                });
                let text = editable.editor().read().rope().to_string();
                *value.write() = text.clone();
                user_text.set(text);
                selected_index.set(-1);
                if !open() {
                    open.set(true);
                }
            }
            _ => {}
        };

        let on_key_up = move |e: Event<KeyboardEventData>| {
            e.stop_propagation();
            editable.process_event(EditableEvent::KeyUp { key: &e.key });
        };

        let on_input_pointer_down = move |e: Event<PointerEventData>| {
            e.stop_propagation();
            is_dragging.set(true);
            if !display_placeholder {
                let area = area.read().to_f64();
                let global_location = e.global_location().clamp(area.min(), area.max());
                let location = (global_location - area.min()).to_point();
                editable.process_event(EditableEvent::Down {
                    location,
                    editor_line: EditorLine::SingleParagraph,
                    holder: &holder.read(),
                });
            }
            focus.request_focus();
            if !open() {
                open.set(true);
            }
        };

        let on_pointer_down = move |e: Event<PointerEventData>| {
            e.stop_propagation();
            is_dragging.set(true);
            if !display_placeholder {
                editable.process_event(EditableEvent::Down {
                    location: e.element_location(),
                    editor_line: EditorLine::SingleParagraph,
                    holder: &holder.read(),
                });
            }
            focus.request_focus();
            if !open() {
                open.set(true);
            }
        };

        let on_global_mouse_move = move |e: Event<MouseEventData>| {
            if focus.is_focused() && *is_dragging.read() {
                let mut location = e.global_location;
                location.x -= area.read().min_x() as f64;
                location.y -= area.read().min_y() as f64;
                editable.process_event(EditableEvent::Move {
                    location,
                    editor_line: EditorLine::SingleParagraph,
                    holder: &holder.read(),
                });
            }
        };

        let on_pointer_enter = move |_| {
            *status.write() = AutoCompleteInputStatus::Hovering;
            if enabled() {
                Cursor::set(CursorIcon::Text);
            } else {
                Cursor::set(CursorIcon::NotAllowed);
            }
        };

        let on_pointer_leave = move |_| {
            if status() == AutoCompleteInputStatus::Hovering {
                Cursor::set(CursorIcon::default());
                *status.write() = AutoCompleteInputStatus::default();
            }
        };

        let on_global_mouse_up = move |_| {
            match *status.read() {
                AutoCompleteInputStatus::Idle if focus.is_focused() => {
                    editable.process_event(EditableEvent::Release);
                }
                AutoCompleteInputStatus::Hovering => {
                    editable.process_event(EditableEvent::Release);
                }
                _ => {}
            };

            if focus.is_focused() {
                if *is_dragging.read() {
                    is_dragging.set(false);
                } else {
                    focus.request_unfocus();
                    open.set(false);
                }
            }
        };

        let a11y_id = focus.a11y_id();

        let (background, cursor_index, text_selection) =
            if enabled() && focus_status() != FocusStatus::Not {
                (
                    theme_colors.hover_background,
                    Some(editable.editor().read().cursor_pos()),
                    editable
                        .editor()
                        .read()
                        .get_visible_selection(EditorLine::SingleParagraph),
                )
            } else {
                (theme_colors.background, None, None)
            };

        let border = if focus_status() == FocusStatus::Keyboard {
            Border::new()
                .fill(theme_colors.focus_border_fill)
                .width(2.)
                .alignment(BorderAlignment::Inner)
        } else {
            Border::new()
                .fill(theme_colors.border_fill)
                .width(1.)
                .alignment(BorderAlignment::Inner)
        };

        let value_text = self.value.read();
        let text = if display_placeholder {
            self.placeholder.as_ref().unwrap().as_ref()
        } else {
            value_text.as_ref()
        };

        let preedit_text = (!display_placeholder)
            .then(|| ime_preedit.read().clone())
            .flatten();

        let (scale, opacity) = animation.read().value();

        rect()
            .child(
                rect()
                    .a11y_id(a11y_id)
                    .a11y_focusable(self.enabled)
                    .a11y_auto_focus(self.auto_focus)
                    .a11y_alt(text.to_string())
                    .a11y_role(AccessibilityRole::TextInput)
                    .maybe(self.enabled, |el| {
                        el.on_key_up(on_key_up)
                            .on_key_down(on_key_down)
                            .on_pointer_down(on_input_pointer_down)
                            .on_ime_preedit(on_ime_preedit)
                            .on_global_mouse_up(on_global_mouse_up)
                            .on_global_mouse_move(on_global_mouse_move)
                    })
                    .on_pointer_enter(on_pointer_enter)
                    .on_pointer_leave(on_pointer_leave)
                    .width(self.width.clone())
                    .background(background.mul_if(!self.enabled, 0.85))
                    .border(border)
                    .corner_radius(theme_layout.corner_radius)
                    .main_align(Alignment::center())
                    .cross_align(Alignment::center())
                    .child(
                        ScrollView::new()
                            .height(Size::Inner)
                            .direction(Direction::Horizontal)
                            .show_scrollbar(false)
                            .child(
                                rect()
                                    .margin(theme_layout.inner_margin)
                                    .maybe(self.enabled, |el| el.on_pointer_down(on_pointer_down))
                                    .child(
                                        paragraph()
                                            .holder(holder.read().clone())
                                            .on_sized(move |e: Event<SizedEventData>| {
                                                area.set(e.visible_area)
                                            })
                                            .min_width(Size::func(move |context| {
                                                Some(
                                                    context.parent
                                                        + theme_layout.inner_margin.horizontal(),
                                                )
                                            }))
                                            .cursor_index(cursor_index)
                                            .color(theme_colors.color)
                                            .cursor_color(theme_colors.color)
                                            .text_align(self.text_align)
                                            .max_lines(1)
                                            .highlights(text_selection.map(|h| vec![h]))
                                            .maybe(display_placeholder, |el| {
                                                el.span(
                                                    self.placeholder.as_ref().unwrap().to_string(),
                                                )
                                                .color(theme_colors.placeholder_color)
                                            })
                                            .maybe(!display_placeholder, |el| {
                                                let mut p = el
                                                    .span(value.read().to_string())
                                                    .color(theme_colors.color);
                                                if let Some(ghost) = ghost_text.clone() {
                                                    p = p
                                                        .span(ghost)
                                                        .color(theme_colors.placeholder_color);
                                                }
                                                if let Some(preedit) = preedit_text {
                                                    p = p.span(preedit).color(theme_colors.color);
                                                }
                                                p
                                            }),
                                    ),
                            ),
                    ),
            )
            .maybe_child((open() && focus.is_focused() && !filtered_suggestions.is_empty()).then(|| {
                rect()
                    .width(Size::Fill)
                    .max_height(Size::px(200.))
                    .margin(Gaps::new(4., 0., 0., 0.))
                    .child(
                        rect()
                            .layer(Layer::Overlay)
                            .border(
                                Border::new()
                                    .fill(theme_colors.border_fill)
                                    .width(1.)
                                    .alignment(BorderAlignment::Inner),
                            )
                            .overflow(Overflow::Clip)
                            .corner_radius(8.)
                            .background(theme_colors.background)
                            .padding(4.)
                            .opacity(opacity)
                            .scale(scale)
                            .expanded()
                            .child(
                                ScrollView::new()
                                    .direction(Direction::Vertical)
                                    .show_scrollbar(true)
                                    .children(filtered_suggestions.iter().enumerate().map(
                                        |(idx, suggestion)| {
                                            let is_selected = selected_index() == idx as i32;
                                            let suggestion = suggestion.clone();
                                            let mut value = value.clone();
                                            let mut editable = editable.clone();

                                            rect()
                                                .width(Size::Fill)
                                                .padding(8.)
                                                .corner_radius(4.)
                                                .background(if is_selected {
                                                    Color::from_rgb(60, 60, 60)
                                                } else {
                                                    Color::TRANSPARENT
                                                })
                                                .on_pointer_enter(move |_| {
                                                    selected_index.set(idx as i32);
                                                })
                                                .on_press({
                                                    let suggestion = suggestion.clone();
                                                    move |_| {
                                                        *value.write() = suggestion.clone();
                                                        editable
                                                            .editor_mut()
                                                            .write()
                                                            .set(&suggestion);
                                                        open.set(false);
                                                        selected_index.set(-1);
                                                    }
                                                })
                                                .child(
                                                    label()
                                                        .color(theme_colors.color)
                                                        .text(suggestion),
                                                )
                                                .into()
                                        },
                                    )),
                            ),
                    )
            }))
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
