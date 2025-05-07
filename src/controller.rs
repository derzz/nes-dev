// If strobe is off, it will read controller input from highest to lowest bit
// If strobe is on, it will read the states from the previous inputs and the state of the first button

use bitflags::bitflags;

bitflags!{
    #[derive(Copy, Clone, Debug)]
    pub struct ControllerButton: u8{
        const RIGHT = 0b1000_0000;
        const LEFT = 0b0100_0000;
        const DOWN = 0b0010_0000;
        const UP = 0b0001_0000;
        const START = 0b0000_1000;
        const SELECT = 0b0000_0100;
        const B = 0b0000_0010;
        const A = 0b0000_0001;
    }
}

pub struct Controller{
    strobe: bool, // Determines if we are writing input or leaving the read
    button_index: u8,
    button_status: ControllerButton
}

impl Controller{
    pub fn new() -> Self {
        Controller{
            strobe: false,
            button_index: 0,
            button_status: ControllerButton::from_bits_truncate(0b0000_0000),
        }
    }

    pub fn write(&mut self, data: u8){
        println!("writing controller!");
        self.strobe = data & 1 == 1;
        if self.strobe{
            println!("resetting strobe!");
            // Starts at the first index
            self.button_index = 0
        }
    }

    pub fn read(&mut self) -> u8{
        println!("reading controller!");
        if self.button_index > 7{
            return 1; // Indicates that there isn't anything left ot read
        }
        let response = (self.button_status.bits() & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7{
            self.button_index += 1; // Increments the index for the next read
        }
        println!("Response is {:x}", response);
        response
    }

    pub fn set_button_pressed_status(&mut self, button: ControllerButton, input: bool){
        println!("Set status to {:?}", button);
        self.button_status.set(button, input);
    }
}
