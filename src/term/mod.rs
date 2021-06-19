pub mod font;
pub mod fullscreen_scroller;
pub mod vertical_scroller;

use ssd1963::{Bounds, Display};

use self::{font::MonoFont, fullscreen_scroller::FullscreenVerticalScroller, vertical_scroller::Scroller};
use core::{
    convert::{TryFrom, TryInto},
    ops::RangeBounds,
};

// pub fn text_to_pixels<'a, 'font: 'a, Font: font::MonoFont>(_font: &'font Font, text: &'a str) -> impl Iterator<Item = bool> + 'a {
//     text.chars().flat_map(move |ch| get_bits(_font, ch))
// }

pub fn get_bits<'font, Font: font::MonoFont>(_font: &'font Font, ch: char) -> impl Iterator<Item = bool> {
    let mut ch = u32::from(ch);
    if ch < 32 || ch > 127 {
        ch = 127
    }
    let ch = ch as u8;
    let bits_per_char = u16::from(Font::CHAR_HEIGHT) * u16::from(Font::CHAR_WIDTH);
    let bit_offset = (u16::from(ch) - 32) * bits_per_char;

    let byte_offset = bit_offset / 8;
    let bit_offset = (bit_offset % 8) as u8;
    CharPixelIter {
        data: &Font::data()[usize::from(byte_offset)..],
        bit_offset,
        count: bits_per_char,
    }
}
pub fn get_bits_transposed<'font, Font: font::MonoFont>(_font: &'font Font, ch: char) -> impl Iterator<Item = bool> + 'font {
    let mut ch = u32::from(ch);
    if ch < 32 || ch > 127 {
        ch = 127
    }
    let ch = ch as u8;
    let bits_per_char = u16::from(Font::CHAR_HEIGHT) * u16::from(Font::CHAR_WIDTH);
    let bit_offset = (u16::from(ch) - 32) * bits_per_char;

    CharPixelTransIter {
        data: &Font::data(),
        bit_offset,
        row: 0,
        col: 0,
        _font,
    }
}

pub struct CharPixelIter {
    data: &'static [u8],
    bit_offset: u8,
    count: u16,
}
impl Iterator for CharPixelIter {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        let bit = self.data[0] & (1 << self.bit_offset);
        if self.bit_offset == 7 {
            self.bit_offset = 0;
            self.data = &self.data[1..];
        } else {
            self.bit_offset += 1;
        }
        Some(if bit == 0 { false } else { true })
    }
}

pub struct CharPixelTransIter<'font, Font> {
    data: &'font [u8],
    bit_offset: u16,
    row: u8,
    col: u8,
    _font: &'font Font,
}
impl<'font, Font: MonoFont> Iterator for CharPixelTransIter<'font, Font> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.row == Font::CHAR_WIDTH {
            self.col += 1;
            if self.col == Font::CHAR_HEIGHT {
                return None;
            }
            self.row = 0;
        }
        let bit_offset = self.bit_offset + u16::from(self.row) * u16::from(Font::CHAR_WIDTH) + u16::from(self.col);
        self.row += 1;
        let byte_offset = bit_offset / 8;
        let bit_offset = bit_offset % 8;
        let bit = self.data[usize::from(byte_offset)] & (1 << bit_offset);
        Some(if bit == 0 { false } else { true })
    }
}

fn display_size<Disp: Display>(_display: &Disp) -> Bounds {
    Bounds {
        x_start: 0,
        x_end: Disp::WIDTH - 1,
        y_start: 0,
        y_end: Disp::HEIGHT - 1,
    }
}

pub struct Term<'me, Disp: Display, Font, Scroller> {
    display: &'me mut Disp,
    font: &'me Font,
    scroller: Scroller,
    bgcolor: Disp::Color,
    fgcolor: Disp::Color,
    bounds: Bounds,
    line_offset: u16,
    column_offset: u16,
    start_with_newline: bool,
}

impl<'me, Disp, Font, Scroll> Term<'me, Disp, Font, Scroll>
where
    Disp: Display<Color = u16>, // TODO: properly implement Color and remove the Color = u16 constrain
    Font: MonoFont,
    Scroll: Scroller<Disp>,
{
    pub fn new(display: &'me mut Disp, font: &'me Font, scroller: Scroll) -> Self {
        Self {
            font,
            scroller,
            bgcolor: 0u16,
            fgcolor: 0b1111111111111111u16,
            bounds: display_size(display),
            display,
            line_offset: 0,
            column_offset: 0,
            start_with_newline: false,
        }
    }
    // panics if requested dimensions are greater than display size
    pub fn dimensions<X, Y>(mut self, x: X, y: Y) -> Self
    where
        X: RangeBounds<u16>,
        Y: RangeBounds<u16>,
    {
        self.bounds = Bounds::new_within(x, y, &display_size(self.display)).unwrap();
        self
    }
    fn scroll_up(&mut self, by: u16) -> Result<(), Disp::Error> {
        let by = -i16::try_from(by).unwrap();
        self.scroller
            .scroll_area(self.display, self.bounds.range_horiz(), self.bounds.range_vert(), 0, by)
    }
    pub fn write(&mut self, text: &str) {
        let line_len = (Disp::WIDTH / u16::from(Font::CHAR_WIDTH)).try_into().unwrap();
        let mut chars = SplitByLenOrNewline::new(text, line_len);

        loop {
            match chars.next() {
                None => return,
                Some(CharOrNewline::NewLine) => self.start_with_newline = true,
                Some(CharOrNewline::Char(c)) => {
                    if self.start_with_newline {
                        let mut remaininig_area = self.bounds.clone();
                        remaininig_area.x_start += self.column_offset;
                        remaininig_area.y_start += self.line_offset;
                        remaininig_area.set_height(u16::from(Font::CHAR_HEIGHT));
                        self.display
                            .fill_area(
                                remaininig_area.range_horiz(),
                                remaininig_area.range_vert(),
                                &mut core::iter::repeat(self.bgcolor),
                            )
                            .ok();

                        // is there space for another line after this one?
                        let remaining_height = self.bounds.height() - self.line_offset - u16::from(Font::CHAR_HEIGHT);
                        self.line_offset = if remaining_height < u16::from(Font::CHAR_HEIGHT) {
                            self.scroll_up(u16::from(Font::CHAR_HEIGHT) - remaining_height).ok();
                            self.bounds.height() - u16::from(Font::CHAR_HEIGHT)
                        } else {
                            self.line_offset + u16::from(Font::CHAR_HEIGHT)
                        };
                        self.start_with_newline = false;
                        self.column_offset = 0;
                    }
                    let (fg, bg) = (self.fgcolor, self.bgcolor);
                    let mut bits = get_bits_transposed(self.font, c).map(move |b| if b { fg } else { bg });
                    let end_column_offset = self.column_offset + u16::from(Font::CHAR_WIDTH);
                    let end_line_offset = self.line_offset + u16::from(Font::CHAR_HEIGHT);
                    let mut abc = self.bounds.clone();
                    abc.x_start += self.column_offset;
                    abc.y_start += self.line_offset;
                    abc.set_height(u16::from(Font::CHAR_HEIGHT));
                    abc.set_width(u16::from(Font::CHAR_WIDTH));
                    self.display.fill_area(abc.range_horiz(), abc.range_vert(), &mut bits).ok();
                    self.column_offset = end_column_offset - 1;
                }
            }
        }
    }
}

impl<'a, Disp, Font, Scroll> core::fmt::Write for Term<'a, Disp, Font, Scroll>
where
    Disp: Display<Color = u16>, // TODO: properly implement Color and remove the Color = u16 constrain
    Font: MonoFont,
    Scroll: Scroller<Disp>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Ok(self.write(s))
    }
}

struct SplitByLenOrNewline<'a> {
    chars: core::str::Chars<'a>,
    line_len: u8,
    line_offset: u8,
}
enum CharOrNewline {
    Char(char),
    NewLine,
}
impl<'a> SplitByLenOrNewline<'a> {
    pub fn new(text: &'a str, line_len: u8) -> Self {
        Self {
            chars: text.chars(),
            line_len,
            line_offset: 0,
        }
    }

    pub fn line_offset(&self) -> u8 {
        self.line_offset
    }
}
impl<'a> Iterator for SplitByLenOrNewline<'a> {
    type Item = CharOrNewline;
    fn next(&mut self) -> Option<Self::Item> {
        if self.line_offset < self.line_len {
            match self.chars.next() {
                None => None,
                Some('\n') | Some('\r') => {
                    self.line_offset = 0;
                    Some(CharOrNewline::NewLine)
                }
                Some(ch) => Some(CharOrNewline::Char(ch)),
            }
        } else {
            self.line_offset = 0;
            Some(CharOrNewline::NewLine)
        }
    }
}
