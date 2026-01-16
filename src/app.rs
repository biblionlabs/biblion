use freya::prelude::*;

pub fn init() -> impl IntoElement {
    let mut theme = use_init_root_theme(|| PreferredTheme::Dark.to_theme());

    let mut search_state = use_state(String::new);

    use_side_effect(move || theme.set(PreferredTheme::Dark.to_theme()));

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
            rect()
                .content(Content::Flex)
                .center()
                .width(Size::fill())
                .expanded()
                .vertical()
                .background(Color::from_rgb(180, 123, 123)),
        )
}
