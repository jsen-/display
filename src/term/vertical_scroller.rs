use core::convert::TryFrom;
use core::{cmp::min, ops::RangeBounds};
use ssd1963::{display::ReadArea, Bounds, Display};

pub trait Scroller<Disp>
where
    Disp: Display,
{
    fn scroll_area<X, Y>(&mut self, disp: &mut Disp, x: X, y: Y, horiz_by: i16, vert_by: i16) -> Result<(), Disp::Error>
    where
        X: RangeBounds<u16>,
        Y: RangeBounds<u16>;
}

pub struct CopyScroller<'a, Disp: Display> {
    buffer: &'a mut [Disp::Color],
}
impl<'a, Disp: ReadArea> CopyScroller<'a, Disp> {
    pub fn new(buffer: &'a mut [Disp::Color]) -> Self {
        Self { buffer }
    }

    fn copy(&mut self, source_window: &Bounds, target_window: &Bounds, disp: &mut Disp) -> Result<(), Disp::Error> {
        let area = usize::try_from(source_window.area()).unwrap();
        let buffer = &mut self.buffer[..area];
        // TODO: make sure the iterator returns at least `area` items
        let fill_buffer = disp
            .read_area(source_window.range_horiz(), source_window.range_vert())?
            .take(area)
            .zip(buffer.iter_mut())
            .find_map(|(item, dest)| match item {
                Ok(color) => {
                    *dest = color;
                    None
                }
                Err(err) => Some(err),
            });
        if let Some(err) = fill_buffer {
            return Err(err);
        }
        disp.fill_area(target_window.range_horiz(), target_window.range_vert(), &mut buffer.iter().copied())
    }
}

impl<'a, Disp: ReadArea> Scroller<Disp> for CopyScroller<'a, Disp> {
    fn scroll_area<X, Y>(&mut self, disp: &mut Disp, x: X, y: Y, horiz_by: i16, vert_by: i16) -> Result<(), Disp::Error>
    where
        X: RangeBounds<u16>,
        Y: RangeBounds<u16>,
    {
        let source = Bounds::new_within(
            x,
            y,
            &Bounds {
                x_start: 0,
                x_end: Disp::WIDTH - 1,
                y_start: 0,
                y_end: Disp::HEIGHT - 1,
            },
        )
        .unwrap();
        let buffer_lines = u16::try_from(min(usize::from(source.height()), self.buffer.len() / usize::from(source.width()))).unwrap();
        let mut source_window = source;
        source_window.set_height(buffer_lines);
        let mut target_window = source_window;
        target_window.move_by(horiz_by, vert_by);

        let iterations = source.height() / buffer_lines;
        for _ in 0..iterations {
            self.copy(&source_window, &target_window, disp)?;
            source_window.move_by(0u16, buffer_lines);
            target_window.move_by(0u16, buffer_lines);
        }
        let remaining_lines = source.height() - iterations * source_window.height();
        if remaining_lines > 0 {
            source_window.set_height(remaining_lines);
            target_window.set_height(remaining_lines);
            self.copy(&source_window, &target_window, disp)?;
        }

        Ok(())
    }
}
