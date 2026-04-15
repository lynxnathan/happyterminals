pub mod style {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum Color {
        #[default]
        Reset,
        Black,
        White,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Modifier(u16);

    impl Modifier {
        pub const fn empty() -> Self {
            Self(0)
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Style {
        pub fg: Color,
        pub bg: Color,
        pub modifier: Modifier,
    }

    impl Style {
        pub fn fg(mut self, color: Color) -> Self {
            self.fg = color;
            self
        }

        pub fn bg(mut self, color: Color) -> Self {
            self.bg = color;
            self
        }

        pub fn add_modifier(mut self, modifier: Modifier) -> Self {
            self.modifier = modifier;
            self
        }
    }
}

