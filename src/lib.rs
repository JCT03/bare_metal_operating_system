#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, peek, plot, plot_str, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
};

use core::{
    clone::Clone, cmp::{min, Eq, PartialEq}, iter::Iterator, marker::Copy, prelude::rust_2024::derive
};

const EDITOR_WIDTH: usize = (BUFFER_WIDTH -3)/2;
const EDITOR_HEIGHT: usize = (BUFFER_HEIGHT -4)/2;
const DOC1_X: usize = 1;
const DOC1_Y: usize = 2;
const DOC2_X: usize = EDITOR_WIDTH + 2;
const DOC2_Y: usize = 2;
const DOC3_X: usize = 1;
const DOC3_Y: usize = EDITOR_HEIGHT+3;
const DOC4_X: usize = EDITOR_WIDTH + 2;
const DOC4_Y: usize = EDITOR_HEIGHT+3;

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Command {
    NewLine,
    BackSpace,
    Left,
    Right,
    Up,
    Down
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct UserInterface {
    docs: [Document; 4],
    current: usize,
    letter_input: Option<char>,
    other_input: Option<Command>
}

impl Default for UserInterface {
    fn default() -> Self {
        Self {
            docs: [Document::new(EDITOR_WIDTH, EDITOR_HEIGHT, DOC1_X, DOC1_Y), 
            Document::new(EDITOR_WIDTH, EDITOR_HEIGHT, DOC2_X, DOC2_Y),
            Document::new(EDITOR_WIDTH, EDITOR_HEIGHT, DOC3_X, DOC3_Y),
            Document::new(EDITOR_WIDTH, EDITOR_HEIGHT, DOC4_X, DOC4_Y)], 
            current: 1, letter_input: None, other_input: None
        }
    }
}

impl UserInterface {
    pub fn tick(&mut self) {
        if peek(BUFFER_WIDTH/2-6, 0) != (' ', ColorCode::new(Color::Blue, Color::White)) {
            self.create_screen(1);
        }
        self.update();
    }
    fn update(&mut self) {
        if self.other_input.is_some() {
            let com = self.other_input.unwrap();
            self.docs[self.current-1].update_cursor(com);
            self.other_input = None;
        }
        if self.letter_input.is_some() {
            let c = self.letter_input.unwrap();
            self.docs[self.current-1].update_document(c);
            self.letter_input = None;
        }
    }
    fn create_screen(&mut self, active: usize) {
        for x in 0..BUFFER_WIDTH {
            for y in 0..BUFFER_HEIGHT {
                if y ==0 {
                    if x ==0 {
                        plot_str(" Text Editor ", BUFFER_WIDTH/2-6, y, ColorCode::new(Color::Blue, Color::White));
                    }
                } else if (y == (DOC1_Y-1)) | (y == (DOC3_Y-1)) | (y == (DOC3_Y+EDITOR_HEIGHT)) | (x == (DOC1_X-1)) | (x == (DOC2_X-1)) | (x == (DOC2_X+EDITOR_WIDTH)){
                    if (y < EDITOR_HEIGHT*2+4) & (x < EDITOR_WIDTH*2+3) {
                        plot(' ', x, y, ColorCode::new(Color::White, Color::Blue));
                    }
                }
            }
        }
        let mut x_doc = DOC1_X;
        let mut y_doc = DOC1_Y;
        if active == 2 {
            x_doc = DOC2_X;
            y_doc = DOC2_Y;
        }
        else if active == 3 {
            x_doc = DOC3_X;
            y_doc = DOC3_Y;
        }
        else if active == 4 {
            x_doc = DOC4_X;
            y_doc = DOC4_Y;
        }
        for x in x_doc-1..(x_doc+EDITOR_WIDTH+1) {
            plot(' ', x, y_doc-1, ColorCode::new(Color::White, Color::LightCyan));
            plot(' ', x, y_doc+EDITOR_HEIGHT, ColorCode::new(Color::White, Color::LightCyan));
        }
        for y in y_doc-1..(y_doc+EDITOR_HEIGHT+1) {
            plot(' ', x_doc-1, y, ColorCode::new(Color::White, Color::LightCyan));
            plot(' ', x_doc+EDITOR_WIDTH, y, ColorCode::new(Color::White, Color::LightCyan));
        }
        plot_str("F1",DOC1_X + (EDITOR_WIDTH/2)-1,DOC1_Y-1,ColorCode::new(Color::White, Color::Blue));
        plot_str("F2",DOC2_X + (EDITOR_WIDTH/2)-1,DOC2_Y-1,ColorCode::new(Color::White, Color::Blue));
        plot_str("F3",DOC3_X + (EDITOR_WIDTH/2)-1,DOC3_Y-1,ColorCode::new(Color::White, Color::Blue));
        plot_str("F4",DOC4_X + (EDITOR_WIDTH/2)-1,DOC4_Y-1,ColorCode::new(Color::White, Color::Blue));
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::Return => {self.other_input = Some(Command::NewLine)}
            KeyCode::ArrowLeft => {self.other_input = Some(Command::Left);}
            KeyCode::ArrowRight => {self.other_input = Some(Command::Right);}
            KeyCode::ArrowUp => {self.other_input = Some(Command::Up);}
            KeyCode::ArrowDown => {self.other_input = Some(Command::Down);}
            KeyCode::F1 => {self.current = 1; self.create_screen(1);}
            KeyCode::F2 => {self.current = 2; self.create_screen(2);}
            KeyCode::F3 => {self.current = 3; self.create_screen(3);}
            KeyCode::F4 => {self.current = 4; self.create_screen(4);}
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        if is_drawable(key) {
            self.letter_input = Some(key);
        } else if key == '\n' {
            self.other_input = Some(Command::NewLine);
        } else if key == '\x08' { //https://www.reddit.com/r/rust/comments/22rgme/what_are_the_supported_escape_sequences_in_rust/?rdt=64438
            self.other_input = Some(Command::BackSpace);
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Document {
    doc_width: usize,
    doc_height: usize,
    top_x: usize,
    top_y: usize,
    cur_x: usize,
    cur_y: usize,
    content: [[char; EDITOR_WIDTH];EDITOR_HEIGHT*4],
    offset: usize
}

impl Document {
    fn new(doc_width: usize, doc_height: usize, top_x: usize, top_y: usize) -> Self {
        plot(' ', top_x, top_y, ColorCode::new(Color::White, Color::White));
        Document {doc_width, doc_height, top_x, top_y, cur_x: 0, cur_y: 0, content: [[' '; EDITOR_WIDTH]; EDITOR_HEIGHT*4], offset:0}
    }
    fn update_cursor(&mut self, c: Command) {
        if c == Command::NewLine && (self.cur_y + 1 < self.doc_height*4) {
            self.cur_x = 0;
            self.cur_y += 1;
        }
        if c == Command::BackSpace {
            if self.cur_x >= 1 {
                self.content[self.cur_y][self.cur_x-1] = ' ';
                self.cur_x -= 1
            } 
            else if self.cur_y >= 1 {
                self.content[self.cur_y-1][self.doc_width -1] = ' ';
                self.cur_y -= 1;
                self.cur_x = self.doc_width -1;
            }
        }
        if c == Command::Left {
            if self.cur_x != 0 {
                self.cur_x -= 1;
            } else {
                self.cur_x = self.doc_width -1;
            }
        }
        if c == Command::Right {
            if self.cur_x != (self.doc_width -1) {
                self.cur_x += 1;
            } else {
                self.cur_x = 0;
            }
        }
        if c == Command::Up {
            if self.cur_y != 0 {
                self.cur_y -= 1;
            }
        }
        if c == Command::Down {
            if self.cur_y != (self.doc_height*4 -1) {
                self.cur_y += 1;
            }
        }
        self.show_document();
    }
    fn update_document(&mut self, c: char) {
        self.content[self.cur_y][self.cur_x] = c;
        if self.cur_x + 1 < self.doc_width {
            self.cur_x += 1;
        } else if self.cur_y + 1 < self.doc_height*4 {
            self.cur_x = 0;
            self.cur_y += 1;
        }
        self.show_document();
    }
    fn show_document(&mut self) {
        if self.cur_y > (self.doc_height + self.offset-1) {
            self.offset += 1;
        }
        if self.cur_y < (self.offset) {
            self.offset -= 1;
        }
        for x in 0..(self.doc_width) {
            for y in 0..(self.doc_height) {
                plot(self.content[y+self.offset][x], x+self.top_x, y+self.top_y, ColorCode::new(Color::White, Color::Black));
            }
        }
        plot(' ', self.cur_x+self.top_x, self.cur_y+self.top_y-self.offset, ColorCode::new(Color::White, Color::White));
    }
}