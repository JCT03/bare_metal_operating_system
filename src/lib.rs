#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, peek, plot, plot_str, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
};

use core::{
    clone::Clone,
    cmp::{min, Eq, PartialEq},
    iter::Iterator,
    marker::Copy,
    prelude::rust_2024::derive,
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
    NewLine
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
        if peek(0, 0) != ('R', ColorCode::new(Color::White, Color::Black)) {
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
                        plot_str("Reader", x, y, ColorCode::new(Color::White, Color::Black));
                    }
                } else if (y == (DOC1_Y-1)) | (y == (DOC3_Y-1)) | (y == (DOC3_Y+EDITOR_HEIGHT)) | (x == (DOC1_X-1)) | (x == (DOC2_X-1)) | (x == (DOC2_X+EDITOR_WIDTH)){
                    if (y < EDITOR_HEIGHT*2+4) & (x < EDITOR_WIDTH*2+3) {
                        plot('.', x, y, ColorCode::new(Color::White, Color::Black));
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
            plot('.', x, y_doc-1, ColorCode::new(Color::White, Color::Blue));
            plot('.', x, y_doc+EDITOR_HEIGHT, ColorCode::new(Color::White, Color::Blue));
        }
        for y in y_doc-1..(y_doc+EDITOR_HEIGHT+1) {
            plot('.', x_doc-1, y, ColorCode::new(Color::White, Color::Blue));
            plot('.', x_doc+EDITOR_WIDTH, y, ColorCode::new(Color::White, Color::Blue));
        }
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
            KeyCode::ArrowLeft => {}
            KeyCode::ArrowRight => {}
            KeyCode::ArrowUp => {}
            KeyCode::ArrowDown => {}
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
            self.other_input = Some(Command::NewLine)
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
}

impl Document {
    fn new(doc_width: usize, doc_height: usize, top_x: usize, top_y: usize) -> Self {
        plot(' ', top_x, top_y, ColorCode::new(Color::White, Color::White));
        Document {doc_width, doc_height, top_x, top_y, cur_x: 0, cur_y: 0}
    }
    fn update_cursor(&mut self, c: Command) {
        if c == Command::NewLine && (self.cur_y + 1 < self.doc_height) {
            plot(' ', self.cur_x+self.top_x, self.cur_y+self.top_y, ColorCode::new(Color::White, Color::Black));
            self.cur_x = 0;
            self.cur_y += 1;
            plot(' ', self.cur_x+self.top_x, self.cur_y+self.top_y, ColorCode::new(Color::White, Color::White));
        }
    }
    fn update_document(&mut self, c: char) {
        plot(c, self.cur_x+self.top_x, self.cur_y+self.top_y, ColorCode::new(Color::White, Color::Black));
        if self.cur_x + 1 < self.doc_width {
            self.cur_x += 1;
        } else if self.cur_y + 1 < self.doc_height {
            self.cur_x = 0;
            self.cur_y += 1;
        }
        plot(' ', self.cur_x+self.top_x, self.cur_y+self.top_y, ColorCode::new(Color::White, Color::White));
    }
}