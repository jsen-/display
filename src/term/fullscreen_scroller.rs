use core::ops::RangeBounds;

use ssd1963::Display;

pub trait FullscreenVerticalScroller<Disp>
where
    Disp: Display,
{
    fn scroll_area<X, Y>(&mut self, disp: &mut Disp, y: Y, vert_by: i16) -> Result<(), Disp::Error>
    where
        X: RangeBounds<u16>,
        Y: RangeBounds<u16>;
}
