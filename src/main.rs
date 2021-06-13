#![no_main]
#![no_std]
#![allow(warnings)]

mod term;

use core::{convert::TryInto, fmt::write, marker::PhantomData, ops::Range};

use cortex_m_rt::entry;

use embedded_hal::digital::v2::OutputPin;
use hal::{
    delay::Delay,
    gpio::gpiob::Parts,
    pac::{CorePeripherals, Peripherals},
    prelude::*,
};
use panic_semihosting as _;
use ssd1963::{display::CopyArea, stm32f1xx::RwPortB, GpioReadWrite16BitInterface, Lcd800x480, Screen};
use stm32f1xx_hal as hal;

use crate::term::{font::ThisFont, text_to_pixels, vertical_scroller::CopyScroller, Term};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = CorePeripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc
        // .cfgr
        // .sysclk(36.mhz())
        // .hclk(36.mhz())
        // .pclk1(36.mhz())
        // .pclk2(64.mhz())
        // .freeze(&mut flash.acr);
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let Parts {
        pb0,
        pb1,
        pb2,
        pb3,
        pb4,
        pb5,
        pb6,
        pb7,
        pb8,
        pb9,
        pb10,
        pb11,
        pb12,
        pb13,
        pb14,
        pb15,
        crl,
        crh,
    } = dp.GPIOB.split(&mut rcc.apb2);
    let (_, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, pb3, pb4);

    let interface = GpioReadWrite16BitInterface::new(
        RwPortB::new(
            pb0, pb1, pb2, pb3, pb4, pb5, pb6, pb7, pb8, pb9, pb10, pb11, pb12, pb13, pb14, pb15, crl, crh,
        ),
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl), // DC
        gpioa.pa2.into_push_pull_output(&mut gpioa.crl), // WR
        gpioa.pa3.into_push_pull_output(&mut gpioa.crl), // RD
    );

    let mut cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl); // LCD_CS
    cs.set_low().unwrap();
    // let mut rd = gpioa.pa3.into_push_pull_output(&mut gpioa.crl); // LCD_RD
    // rd.set_high().unwrap();
    // let mut reset = gpioa.pa0.into_push_pull_output(&mut gpioa.crl); // LCD_RESET
    // reset.set_high().unwrap();
    let mut disp = ssd1963::Ssd1963::new(ssd1963::Lcd800x480, interface, Delay::new(cp.SYST, clocks)).unwrap();

    struct Gradient<Lcd: Screen> {
        line: u16,
        col: u16,
        _p: PhantomData<Lcd>,
    }

    impl<Lcd: Screen> Gradient<Lcd> {
        pub fn new() -> Self {
            Self {
                line: 0,
                col: 0,
                _p: PhantomData,
            }
        }
    }

    impl<Lcd: Screen> Iterator for Gradient<Lcd> {
        type Item = u16;
        fn next(&mut self) -> Option<Self::Item> {
            self.col = if self.col == Lcd::WIDTH - 1 {
                self.line += 1;
                if self.line == Lcd::HEIGHT - 1 {
                    return None;
                }
                0
            } else {
                self.col + 1
            };
            const RED_MAX: u16 = 0b11111;
            const GREEN_MAX: u16 = 0b111111;
            const BLUE_MAX: u16 = 0b11111;

            let red = self.line / ((Lcd::HEIGHT + RED_MAX) / (RED_MAX + 1)) << 11;
            let green = self.line / ((Lcd::HEIGHT + GREEN_MAX) / (GREEN_MAX + 1)) << 5;
            let blue = self.line / ((Lcd::HEIGHT + BLUE_MAX) / (BLUE_MAX + 1)) << 0;
            Some(red | green | blue)
        }
    }
    use ssd1963::Display;
    let mut buffer = [0u16; 8000];

    // let mut it = Gradient::<Lcd800x480>::new();

    // disp.fill_area(0..disp.width(), 0..disp.height(), &mut it).unwrap();

    // disp.copy_area(0..100, 100..479, 100, -100, &mut buffer).unwrap();
    // disp.fill_area_color(0..480, 380..=380, 0b11111100000);

    // let mut x: u16 = 0;
    // let mut y: u16 = 0;
    // let mut speed_x: i16 = 1;
    // let mut speed_y: i16 = 1;
    // let width: u16 = 100;
    // let height: u16 = 100;
    // use core::convert::TryFrom;
    disp.fill_area_color(.., .., 0).unwrap();

    let scroller = CopyScroller::new(&mut buffer);
    let mut term = Term::new(&mut disp, &ThisFont, scroller).dimensions(.., 8..);
    use core::fmt::Write;
    for i in 0..100 {
        writeln!(&mut term, "{:3} Hello, world!", i);
    }

    // let mut x: u16 = 0;
    // let mut y: u16 = 0;
    // let mut speed_x: i16 = 1;
    // let mut speed_y: i16 = 1;
    // let width: u16 = 100;
    // let height: u16 = 100;
    // use core::convert::TryFrom;
    // loop {
    //     disp.fill_area_color(x..x + width, y..y + width, 0b1111111111111111).unwrap();
    //     let mut it = text_to_pixels(&term::font::ThisFont, "Shupaci!!!").map(|b| if b { 0 } else { 0b1111111111111111 });
    //     disp.fill_area(x + 10..x + 10 + 9 * 8, y + 46..y + 46 + 8, &mut it).unwrap();
    //     for _ in 0..98 {
    //         disp.delay.delay_us(255u8);
    //     }
    //     disp.delay.delay_us(165u8);
    //     disp.fill_area_color(x..x + width, y..y + width, 0).unwrap();
    //     x = (i16::try_from(x).unwrap() + speed_x).try_into().unwrap();
    //     y = (i16::try_from(y).unwrap() + speed_y).try_into().unwrap();
    //     if x == 0 {
    //         speed_x = 1;
    //     } else if x + width == disp.width() {
    //         speed_x = -1;
    //     }
    //     if y == 0 {
    //         speed_y = 1;
    //     } else if y + height == disp.height() {
    //         speed_y = -1;
    //     }
    // }
    loop {}
}
