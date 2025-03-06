use std::collections::HashMap;

pub enum CurrentScreen {
    Main,    // Main summary screen
    Exiting, // Asking if user wants to output key-value pairs they have entered
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

pub struct App {
    pub current_page: u8,
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>, // the optional state containing which of the key or value pair the user is editing. It is an option, because when the user is not directly editing a key-value pair, this will be set to `None`.
}

impl App {
    pub fn new() -> App {
        App {
            current_page: 0x00,
            current_screen: CurrentScreen::Main,
            currently_editing: None,
        }
    }

    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::Key => self.currently_editing = Some(CurrentlyEditing::Value),
                CurrentlyEditing::Value => self.currently_editing = Some(CurrentlyEditing::Key),
            };
        } else {
            self.currently_editing = Some(CurrentlyEditing::Key);
        }
    }

    // pub fn print_json(&self) -> serde_json::Result<()> {
    //     let output = serde_json::to_string(&self.pairs)?;
    //     println!("{}", output);
    //     Ok(())
    // }
}
