use core::ops::RangeBounds;

use ssd1963::{display::CopyArea, Display};

pub trait VerticalScroller<Disp>
where
    Disp: Display,
{
    fn scroll_area<X, Y>(&mut self, disp: &mut Disp, x: X, y: Y, horiz_by: i16, vert_by: i16) -> Result<(), Disp::Error>
    where
        X: RangeBounds<u16>,
        Y: RangeBounds<u16>;
}

pub struct CopyScroller<'a, Disp: Display>(&'a mut [Disp::Color]);
impl<'a, Disp: Display> CopyScroller<'a, Disp> {
    pub fn new(buffer: &'a mut [Disp::Color]) -> Self {
        Self(buffer)
    }
}

impl<'a, Disp: CopyArea> VerticalScroller<Disp> for CopyScroller<'a, Disp> {
    fn scroll_area<X, Y>(&mut self, disp: &mut Disp, x: X, y: Y, horiz_by: i16, vert_by: i16) -> Result<(), Disp::Error>
    where
        X: RangeBounds<u16>,
        Y: RangeBounds<u16>,
    {
        disp.copy_area(x, y, horiz_by, vert_by, self.0)
    }
}
