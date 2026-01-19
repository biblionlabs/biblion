use freya::prelude::*;

/// ToolbarItem representa un botón en la toolbar que al pulsarlo abrirá un ContextMenu.
#[derive(Clone, PartialEq)]
pub struct ToolbarItem {
    pub label: ReadState<String>,
    pub menu: Menu,
    pub key: DiffKey,
}

impl ToolbarItem {
    pub fn new(label: impl Into<ReadState<String>>, menu: Menu) -> Self {
        Self {
            label: label.into(),
            menu,
            key: DiffKey::None,
        }
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for ToolbarItem {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for ToolbarItem {
    fn render(&self) -> impl IntoElement {
        let menu = self.menu.clone();
        ButtonSegment::new()
            .on_press(move |_| {
                ContextMenu::open(menu.clone());
            })
            .child(label().text(self.label.read().to_string()))
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}

/// Toolbar es un componente que organiza varios ToolbarItem en una barra horizontal,
/// estilo barra de herramientas de Windows.
#[derive(Clone, PartialEq)]
pub struct Toolbar {
    children: Vec<Element>,
    key: DiffKey,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            children: vec![],
            key: DiffKey::None,
        }
    }

    pub fn child(mut self, el: impl Into<Element>) -> Self {
        self.children.push(el.into());
        self
    }
}

impl ChildrenExt for Toolbar {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl KeyExt for Toolbar {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for Toolbar {
    fn render(&self) -> impl IntoElement {
        let theme = get_theme_or_default();
        let theme = theme.read();
        let mut requested_theme = theme.segmented_button.clone();

        let SegmentedButtonTheme { border_fill, .. } = requested_theme.resolve(&theme.colors);

        rect()
            .width(Size::Fill)
            .overflow(Overflow::Clip)
            .font_size(14.)
            .border(
                Border::new()
                    .fill(border_fill)
                    .width(1.)
                    .alignment(BorderAlignment::Outer),
            )
            .horizontal()
            .children(self.children.clone())
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
