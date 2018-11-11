use midi::MidiValue;

//XXX move to its own crate

const PAD_BYTES: usize = 64;
const SLIDER_BYTES: usize = 9;
const ROTARY_BYTES: usize = 2;
const BUTTON_BYTES: usize = 15;
const RHOMBUS_BYTES: usize = 2;
const DISPLAY_BYTES: usize = PAD_BYTES + SLIDER_BYTES + ROTARY_BYTES + BUTTON_BYTES + RHOMBUS_BYTES;

pub struct QuNeoDisplay {
    next: [u8; DISPLAY_BYTES],
    last: [u8; DISPLAY_BYTES],
    pad_channel: u8,
    slider_channel: u8,
    rotary_channel: u8,
    button_channel: u8,
    rhombus_channel: u8,
}

#[derive(Copy, Clone, Debug)]
pub enum DisplayType {
    Pad,
    Slider,
    Rotary,
    Button,
    Rhombus,
}

pub struct QuNeoDisplayIter<'a> {
    display: &'a mut QuNeoDisplay,
    index: usize,
}

impl<'a> Iterator for QuNeoDisplayIter<'a> {
    type Item = MidiValue;

    fn next(&mut self) -> Option<MidiValue> {
        while self.index < DISPLAY_BYTES
            && self.display.last[self.index] == self.display.next[self.index]
        {
            self.index += 1;
        }

        let mut response = None;
        if self.index < DISPLAY_BYTES
            && self.display.last[self.index] != self.display.next[self.index]
        {
            response = self.display.value(self.index);
            self.display.last[self.index] = self.display.next[self.index];
            self.index += 1;
        }
        response
    }
}

impl QuNeoDisplay {
    pub fn new() -> Self {
        Self {
            next: [0; DISPLAY_BYTES],
            last: [0; DISPLAY_BYTES],
            pad_channel: 15,
            slider_channel: 14,
            rotary_channel: 14,
            button_channel: 14,
            rhombus_channel: 14,
        }
    }

    pub fn clear(&mut self) {
        self.next = [0; DISPLAY_BYTES]
    }

    pub fn force_draw(&mut self) {
        self.last = [0xFF; DISPLAY_BYTES];
    }

    fn byte_offset(&self, display: DisplayType) -> usize {
        match display {
            DisplayType::Pad => 0,
            DisplayType::Slider => PAD_BYTES,
            DisplayType::Rotary => PAD_BYTES + SLIDER_BYTES,
            DisplayType::Button => PAD_BYTES + SLIDER_BYTES + ROTARY_BYTES,
            DisplayType::Rhombus => PAD_BYTES + SLIDER_BYTES + ROTARY_BYTES + BUTTON_BYTES,
        }
    }

    fn byte_len(&self, display: DisplayType) -> usize {
        match display {
            DisplayType::Pad => PAD_BYTES,
            DisplayType::Slider => SLIDER_BYTES,
            DisplayType::Rotary => ROTARY_BYTES,
            DisplayType::Button => BUTTON_BYTES,
            DisplayType::Rhombus => RHOMBUS_BYTES,
        }
    }

    pub fn update(&mut self, display: DisplayType, index: usize, value: u8) {
        let index = self.byte_offset(display) + num::clamp(index, 0, self.byte_len(display));
        self.next[index] = num::clamp(value, 0, 127);
    }

    fn value(&self, index: usize) -> Option<MidiValue> {
        let value = self.next[index];
        let mut d = None;
        let mut display_index: u8 = 0;

        for display in &[
            DisplayType::Pad,
            DisplayType::Slider,
            DisplayType::Rotary,
            DisplayType::Button,
            DisplayType::Rhombus,
        ] {
            let off = self.byte_offset(*display);
            if index < off + self.byte_len(*display) {
                d = Some(display);
                display_index = (index - off) as u8;
                break;
            }
        }

        let mut v = None;
        match d {
            None => (),
            Some(DisplayType::Pad) => {
                v = Some(MidiValue::Note {
                    on: true,
                    chan: self.pad_channel,
                    vel: value,
                    num: remap_pad(display_index),
                });
            }
            Some(DisplayType::Slider) => {
                v = Some(MidiValue::ContCtrl {
                    chan: self.slider_channel,
                    num: remap_slider(display_index),
                    val: value,
                });
            }
            Some(DisplayType::Rotary) => {
                v = Some(MidiValue::ContCtrl {
                    chan: self.rotary_channel,
                    num: remap_rotary(display_index),
                    val: value,
                });
            }
            Some(DisplayType::Button) => {
                v = Some(MidiValue::Note {
                    on: true,
                    chan: self.button_channel,
                    vel: value,
                    num: remap_button(display_index),
                });
            }
            Some(DisplayType::Rhombus) => {
                v = Some(MidiValue::Note {
                    on: true,
                    chan: self.rhombus_channel,
                    vel: value,
                    num: remap_rhombus(display_index),
                });
            }
        }

        v
    }

    pub fn draw_iter(&mut self) -> QuNeoDisplayIter {
        QuNeoDisplayIter {
            display: self,
            index: 0,
        }
    }
}

impl Default for QuNeoDisplay {
    fn default() -> Self {
        Self::new()
    }
}

fn remap_pad(num: u8) -> u8 {
    let bank = num / 8;
    let off = num % 8;
    2 * ((7 - bank) * 8 + off)
}

fn remap_slider(num: u8) -> u8 {
    let vals = [11, 10, 9, 8, 1, 2, 3, 4, 5];
    vals[num as usize]
}

fn remap_rotary(num: u8) -> u8 {
    let vals = [6, 7];
    vals[num as usize]
}

fn remap_button(num: u8) -> u8 {
    let vals = [
        33, 34, 35, //diamond, square, arrow
        36, 37, 38, 39, 40, 41, 42, 43, // left, right
        46, 47, 48, 49, //up, down
    ];
    vals[num as usize]
}

fn remap_rhombus(num: u8) -> u8 {
    let vals = [44, 45];
    vals[num as usize]
}
