use std::collections::HashMap;

pub enum CurrentScreen {
    Main,    // Main summary screen
    Exiting, // Asking if user wants to output key-value pairs they have entered
}


pub struct App {
    pub current_page: u8,
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub terminal_input: String,
    pub editing: bool
}

impl App {
    pub fn new() -> App {
        App {
            current_page: 0x00,
            current_screen: CurrentScreen::Main,            
            terminal_input: String::new(),
            editing: false
        }
    }

}
