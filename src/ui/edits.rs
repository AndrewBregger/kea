pub enum Operation {
    ScrollUp,
    ScrollDown,
    // InsertChar,
    CursorDown,
    CursorUp,
    CursorLeft,
    CursorRight,
}

impl std::str::FromStr for Operation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scroll_up" => Ok(Self::ScrollUp),
            "scroll_down" => Ok(Self::ScrollDown),
            "cursor_down" => Ok(Self::CursorDown),
            "cursor_up" => Ok(Self::CursorUp),
            "cursor_left" => Ok(Self::CursorLeft),
            "cursor_right" => Ok(Self::CursorRight),
            _ => Err(()),
        }
    }
}
