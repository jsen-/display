#![no_main]
#![no_std]
#![allow(non_snake_case)]
#![feature(llvm_asm)]

use core::{convert::Infallible, mem::size_of};

use cortex_m_rt::entry;

use embedded_hal::{blocking::delay::DelayUs, digital::v2::OutputPin};
use gpiob::{Parts, PB0, PB1, PB15, PB2, PB3, PB4, PB5, PB6, PB7};
use hal::{
    delay::Delay,
    gpio::{
        gpiob::{self, PB10, PB11, PB12, PB13, PB14, PB8, PB9},
        Dynamic, Output, PinMode, PushPull,
    },
    pac::{CorePeripherals, Peripherals},
    prelude::*,
};
use panic_semihosting as _;
use stm32f1xx_hal as hal;

pub struct MCU8080_8Bit<'a, DC, WR, RD, Delay, CS, RESET> {
    pub data: (
        PB0<Dynamic>,
        PB1<Dynamic>,
        PB2<Dynamic>,
        PB3<Dynamic>,
        PB4<Dynamic>,
        PB5<Dynamic>,
        PB6<Dynamic>,
        PB7<Dynamic>,
        PB8<Dynamic>,
        PB9<Dynamic>,
        PB10<Dynamic>,
        PB11<Dynamic>,
        PB12<Dynamic>,
        PB13<Dynamic>,
        PB14<Dynamic>,
        PB15<Dynamic>,
    ),
    pub dc: DC, // high for data, low for command
    pub wr: WR, // low for write
    pub rd: RD, // low for read
    pub crl: &'a mut gpiob::CRL,
    pub crh: &'a mut gpiob::CRH,
    pub cs: CS,
    pub reset: RESET,
    pub delay: Delay,
}

trait ToBeBytes<const B: usize>: Sized {
    fn to_be_bytes(self) -> [u8; B];
}
macro_rules! impl_to_be_bytes {
    ($ty:ty) => {
        impl ToBeBytes<{ ::core::mem::size_of::<Self>() }> for $ty {
            fn to_be_bytes(self) -> [u8; ::core::mem::size_of::<Self>()] {
                self.to_be_bytes()
            }
        }
    };
    ($($ty:ty),+) => {
        $(impl_to_be_bytes!($ty);)+
    };
}
impl_to_be_bytes!(u8, u16, u32, u64, u128, usize);

fn nth_least_significant_byte<T: ToBeBytes<B>, const B: usize>(n: u8, u: T) -> u8 {
    if n as usize >= size_of::<T>() {
        0
    } else {
        u.to_be_bytes()[B - 1 - n as usize]
    }
}

impl<'a, DC, WR, RD, Delay, CS, RESET> MCU8080_8Bit<'a, DC, WR, RD, Delay, CS, RESET>
where
    DC: OutputPin<Error = Infallible>,
    WR: OutputPin<Error = Infallible>,
    RD: OutputPin<Error = Infallible>,
    CS: OutputPin<Error = Infallible>,
    RESET: OutputPin<Error = Infallible>,
    Delay: DelayUs<u32>,
{
    // fn dir_read(&mut self) {
    //     self.data.0.make_floating_input(self.crl);
    //     self.data.1.make_floating_input(self.crl);
    //     self.data.2.make_floating_input(self.crl);
    //     self.data.3.make_floating_input(self.crl);
    //     self.data.4.make_floating_input(self.crl);
    //     self.data.5.make_floating_input(self.crl);
    //     self.data.6.make_floating_input(self.crl);
    //     self.data.7.make_floating_input(self.crl);
    //     self.data.8.make_floating_input(self.crh);
    //     self.data.9.make_floating_input(self.crh);
    //     self.data.10.make_floating_input(self.crh);
    //     self.data.11.make_floating_input(self.crh);
    //     self.data.12.make_floating_input(self.crh);
    //     self.data.13.make_floating_input(self.crh);
    //     self.data.14.make_floating_input(self.crh);
    //     self.data.15.make_floating_input(self.crh);
    // }
    pub fn dir_write(&mut self) {
        self.data.0.make_push_pull_output(self.crl);
        self.data.1.make_push_pull_output(self.crl);
        self.data.2.make_push_pull_output(self.crl);
        self.data.3.make_push_pull_output(self.crl);
        self.data.4.make_push_pull_output(self.crl);
        self.data.5.make_push_pull_output(self.crl);
        self.data.6.make_push_pull_output(self.crl);
        self.data.7.make_push_pull_output(self.crl);
        self.data.8.make_push_pull_output(self.crh);
        self.data.9.make_push_pull_output(self.crh);
        self.data.10.make_push_pull_output(self.crh);
        self.data.11.make_push_pull_output(self.crh);
        self.data.12.make_push_pull_output(self.crh);
        self.data.13.make_push_pull_output(self.crh);
        self.data.14.make_push_pull_output(self.crh);
        self.data.15.make_push_pull_output(self.crh);
    }
    pub fn set_value(&mut self, value: u16) {
        unsafe { (&*stm32f1::stm32f103::GPIOB::ptr()).odr.write(|w| w.bits(value as u32)) };
        #[cfg(not(feature = "prod"))]
        self.delay.delay_us(1);
        #[cfg(feature = "prod")]
        for _ in 0..51 { // 45 was the minimum number that worked
            cortex_m::asm::nop();
        }
        // macro_rules! set_bit {
        //     ($index: tt) => {
        //         if value & (1 << $index) == 0 {
        //             self.data.$index.set_low().unwrap();
        //         } else {
        //             self.data.$index.set_high().unwrap();
        //         }
        //     };
        //     ($($index:tt),+) => {
        //         $(set_bit!($index);)+
        //     };
        // }
        // set_bit!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    }

    // fn get_value(&mut self) -> u16 {
    //     let mut value = 0;
    //     macro_rules! get_bit {
    //         ($index: tt) => {
    //             if self.data.$index.is_high().unwrap() {
    //                 value |= (1 >> 0);
    //             }
    //         };
    //         ($($index:tt),+) => {
    //             $(get_bit!($index);)+
    //         };
    //     }
    //     get_bit!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    //     value
    // }
    const WIDTH: u16 = 800;
    const HEIGHT: u16 = 480;
    pub fn init(&mut self) {
        self.dir_write();
        self.rd.set_high().unwrap();
        self.wr.set_high().unwrap();
        self.cs.set_low().unwrap();

        self.write_command(0xE2); // SSD1963_SET_PLL_MN
        self.write_data(0x1E);
        self.write_data(0x02);
        self.write_data(0x54);

        self.write_command(0xE0); // SSD1963_SET_PLL
        self.write_data(1);
        self.delay.delay_us(100); // PLL lock
        self.write_command(0xE0); // SSD1963_SET_PLL again
        self.write_data(3);

        self.write_command(0x01); // SSD1963_SOFT_RESET

        let fps: u64 = 30;
        let hsync_back_porch: u64 = 3;
        let hsync_front_porch: u64 = 0;
        let hsync_pulse: u64 = 0;
        let vsync_back_porch: u64 = 10;
        let vsync_front_porch: u64 = 0;
        let vsync_pulse: u64 = 0;
        let hsync_period: u64 = hsync_pulse + hsync_back_porch + Self::WIDTH as u64 + hsync_front_porch;
        let vsync_period: u64 = vsync_pulse + vsync_back_porch + Self::HEIGHT as u64 + vsync_front_porch;
        let pclk: u64 = hsync_period * vsync_period * fps;
        let fpr: u64 = pclk * 1048576 / 100000000;
        self.write_command(0xE6); // SSD1963_SET_LSHIFT_FREQ
        self.write_data(nth_least_significant_byte(2, fpr) as u16);
        self.write_data(nth_least_significant_byte(1, fpr) as u16);
        self.write_data(nth_least_significant_byte(0, fpr) as u16);

        self.write_command(0xB0); // SSD1963_SET_LCD_MODE
        self.write_data(0x24);
        self.write_data(0x00);
        self.write_data(nth_least_significant_byte(1, Self::WIDTH) as u16);
        self.write_data(nth_least_significant_byte(0, Self::WIDTH) as u16);
        self.write_data(nth_least_significant_byte(1, Self::HEIGHT) as u16);
        self.write_data(nth_least_significant_byte(0, Self::HEIGHT) as u16);
        self.write_data(0);

        self.write_command(0xB4); // SSD1963_SET_HORI_PERIOD
        self.write_data(0x03);
        self.write_data(0xA0);
        self.write_data(0x00);
        self.write_data(0x2E);
        self.write_data(0x30);
        self.write_data(0x00);
        self.write_data(0x0F);
        self.write_data(0x00);

        self.write_command(0xB6); // SSD1963_SET_VERT_PERIOD
        self.write_data(0x02);
        self.write_data(0x0D);
        self.write_data(0x00);
        self.write_data(0x10);
        self.write_data(0x10);
        self.write_data(0x00);
        self.write_data(0x08);

        self.write_command(0xBA); // SSD1963_SET_GPIO_VALUE
        self.write_data(0x0F);

        self.write_command(0xB8); // SSD1963_SET_GPIO_CONF
        self.write_data(0x07);
        self.write_data(0x01);

        self.write_command(0x36); // SSD1963_SET_ADDRESS_MODE
        self.write_data(0x22);

        self.write_command(0xF0); // SSD1963_SET_PIXEL_DATA_INTERFACE
        self.write_data(0x03); // SSD1963_PDI_16BIT565

        self.write_command(0xBE); // SSD1963_SET_PWM_CONF
        self.write_data(0x06);
        self.write_data(0xf0);
        self.write_data(0x01);
        self.write_data(0xf0);
        self.write_data(0x00);
        self.write_data(0x00);

        self.write_command(0xd0); // SSD1963_SET_DBC_CONF
        self.write_data(0x0d);

        self.clear_screen(0b0000011111100000);
        self.write_command(0x29); // SSD1963_SET_DISPLAY_ON

        self.cs.set_high().unwrap();
    }

    pub fn write_command(&mut self, b: u16) {
        self.dc.set_low().unwrap();
        self.set_value(b);
        self.wr.set_low().unwrap();
        self.wr.set_high().unwrap();
    }
    pub fn write_data(&mut self, b: u16) {
        self.dc.set_high().unwrap();
        self.set_value(b);
        self.wr.set_low().unwrap();
        self.wr.set_high().unwrap();
    }
    pub fn delay(&mut self, ms: u32) {
        self.delay.delay_us(ms);
    }
    // fn read_data(&mut self) -> u16 {
    //     self.dc.set_low().unwrap();

    //     self.rd.set_low().unwrap();
    //     let ret = self.get_value();
    //     self.rd.set_high().unwrap();
    //     ret
    // }

    pub fn set_area(&mut self, sx: u16, ex: u16, sy: u16, ey: u16) {
        self.write_command(0x2A); // SSD1963_SET_COLUMN_ADDRESS
        self.write_data(nth_least_significant_byte(1, sx) as u16);
        self.write_data(nth_least_significant_byte(0, sx) as u16);
        self.write_data(nth_least_significant_byte(1, ex) as u16);
        self.write_data(nth_least_significant_byte(0, ex) as u16);

        self.write_command(0x2B); // SSD1963_SET_PAGE_ADDRESS
        self.write_data(nth_least_significant_byte(1, sy) as u16);
        self.write_data(nth_least_significant_byte(0, sy) as u16);
        self.write_data(nth_least_significant_byte(1, ey) as u16);
        self.write_data(nth_least_significant_byte(0, ey) as u16);
    }

    pub fn fill_area(&mut self, sx: u16, ex: u16, sy: u16, ey: u16, color: u16) {
        self.set_area(sx, ex, sy, ey);
        self.write_command(0x2C); // SSD1963_WRITE_MEMORY_START
        for _i in 0..(ex - sx + 1) as usize * (ey - sy + 1) as usize {
            self.write_data(color);
        }
    }
    pub fn clear_screen(&mut self, color: u16) {
        let __black = 0b0000000000000000;
        let ____red = 0b1111100000000000;
        let _yellow = 0b1111111111100000;
        let __green = 0b0000011111100000;
        let ___cyan = 0b0000011111111111;
        let ___blue = 0b0000000000011111;
        let __white = 0b1111111111111111;
        self.fill_area(0, Self::WIDTH, 0, Self::HEIGHT, color);
    }

    // fn write_something(&mut self) -> (u8, u8, u8, u8) {
    //     for i in 0..50 {
    //         self.set_value(i);
    //         self.wr.set_low().unwrap();
    //         nop();
    //         self.wr.set_high().unwrap();
    //     }

    //     // self.dir_read();
    //     // self.rdx.set_low().unwrap();
    //     // let b0 = self.get_value();
    //     // self.rdx.set_high().unwrap();

    //     // self.rdx.set_low().unwrap();
    //     // let b1 = self.get_value();
    //     // self.rdx.set_high().unwrap();

    //     // self.rdx.set_low().unwrap();
    //     // let b2 = self.get_value();
    //     // self.rdx.set_high().unwrap();

    //     // self.rdx.set_low().unwrap();
    //     // let b3 = self.get_value();
    //     // self.rdx.set_high().unwrap();

    //     self.cs.set_high().unwrap();

    //     // (b0, b1, b2, b3)
    //     (0, 0, 0, 0)
    // }
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = CorePeripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc
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
        mut crl,
        mut crh,
    } = dp.GPIOB.split(&mut rcc.apb2);
    let (_pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, pb3, pb4);

    let p0 = pb0.into_dynamic(&mut crl);
    let p1 = pb1.into_dynamic(&mut crl);
    let p2 = pb2.into_dynamic(&mut crl);
    let p3 = pb3.into_dynamic(&mut crl);
    let p4 = pb4.into_dynamic(&mut crl);
    let p5 = pb5.into_dynamic(&mut crl);
    let p6 = pb6.into_dynamic(&mut crl);
    let p7 = pb7.into_dynamic(&mut crl);
    let p8 = pb8.into_dynamic(&mut crh);
    let p9 = pb9.into_dynamic(&mut crh);
    let p10 = pb10.into_dynamic(&mut crh);
    let p11 = pb11.into_dynamic(&mut crh);
    let p12 = pb12.into_dynamic(&mut crh);
    let p13 = pb13.into_dynamic(&mut crh);
    let p14 = pb14.into_dynamic(&mut crh);
    let p15 = pb15.into_dynamic(&mut crh);
    unsafe {
        PB8::<Output<PushPull>>::set_mode(&mut crh);
    }

    let mut x = MCU8080_8Bit {
        data: (p0, p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11, p12, p13, p14, p15),
        dc: gpioa.pa1.into_push_pull_output(&mut gpioa.crl), // LCD_RS
        wr: gpioa.pa2.into_push_pull_output(&mut gpioa.crl), // LCD_WR
        rd: gpioa.pa3.into_push_pull_output(&mut gpioa.crl), // LCD_RD
        crl: &mut crl,
        crh: &mut crh,
        delay: Delay::new(cp.SYST, clocks),
        cs: gpioa.pa4.into_push_pull_output(&mut gpioa.crl),    // LCD_CS
        reset: gpioa.pa0.into_push_pull_output(&mut gpioa.crl), // LCD_RESET
    };
    x.dir_write();
    x.rd.set_high().unwrap();
    x.wr.set_high().unwrap();
    x.cs.set_low().unwrap();

    // x.write_command(0xE2);		//PLL multiplier, set PLL clock to 120M
    // x.write_data(0x23);	    //N=0x36 for 6.5M, 0x23 for 10M crystal
    // x.write_data(0x02);
    // x.write_data(0x54);
    // x.write_command(0xE0);		// PLL enable
    // x.write_data(0x01);
    // x.delay(10);
    // x.write_command(0xE0);
    // x.write_data(0x03);
    // x.delay(10);
    // x.write_command(0x01);		// software reset
    // x.delay(100);
    // x.write_command(0xE6);		//PLL setting for PCLK, depends on resolution
    // x.write_data(0x01);
    // x.write_data(0x1F);
    // x.write_data(0xFF);
    // x.write_command(0xB0);		//LCD SPECIFICATION
    // x.write_data(0x20);
    // x.write_data(0x00);
    // x.write_data(0x01);		//Set HDP	479
    // x.write_data(0xDF);
    // x.write_data(0x01);		//Set VDP	271
    // x.write_data(0x0F);
    // x.write_data(0x00);
    // x.write_command(0xB4);		//HSYNC
    // x.write_data(0x02);		//Set HT	531
    // x.write_data(0x13);
    // x.write_data(0x00);		//Set HPS	8
    // x.write_data(0x08);
    // x.write_data(0x2B);		//Set HPW	43
    // x.write_data(0x00);		//Set LPS	2
    // x.write_data(0x02);
    // x.write_data(0x00);
    // x.write_command(0xB6);		//VSYNC
    // x.write_data(0x01);		//Set VT	288
    // x.write_data(0x20);
    // x.write_data(0x00);		//Set VPS	4
    // x.write_data(0x04);
    // x.write_data(0x0c);		//Set VPW	12
    // x.write_data(0x00);		//Set FPS	2
    // x.write_data(0x02);
    // x.write_command(0xBA);
    // x.write_data(0x0F);		//GPIO[3:0] out 1
    // x.write_command(0xB8);
    // x.write_data(0x07);	    //GPIO3=input, GPIO[2:0]=output
    // x.write_data(0x01);		//GPIO0 normal
    // x.write_command(0x36);		//rotation
    // x.write_data(0x2A);
    // x.write_command(0xF0);		//pixel data interface
    // x.write_data(0x03);
    // x.delay(1);
    // x.write_command(0xB8);
    // x.write_data(0x0f);    //GPIO is controlled by host GPIO[3:0]=output   GPIO[0]=1  LCD ON  GPIO[0]=1  LCD OFF
    // x.write_data(0x01);    //GPIO0 normal
    // x.write_command(0xBA);
    // x.write_data(0x01);    //GPIO[0] out 1 --- LCD display on/off control PIN
    // // setXY(0, 0, 479, 271);
    // x.write_command(0x29);		//display on
    // x.write_command(0xBE);		//set PWM for B/L
    // x.write_data(0x06);
    // x.write_data(0xf0);
    // x.write_data(0x01);
    // x.write_data(0xf0);
    // x.write_data(0x00);
    // x.write_data(0x00);
    // x.write_command(0xd0);
    // x.write_data(0x0d);
    // x.write_command(0x2C);

    x.write_command(0xE2); //PLL multiplier, set PLL clock to 120M
    x.write_data(0x1E); //N=0x36 for 6.5M, 0x23 for 10M crystal
    x.write_data(0x02);
    x.write_data(0x54);
    x.write_command(0xE0); // PLL enable
    x.write_data(0x01);
    x.delay(10);
    x.write_command(0xE0);
    x.write_data(0x03);
    x.delay(10);
    x.write_command(0x01); // software reset
    x.delay(100);
    x.write_command(0xE6); //PLL setting for PCLK, depends on resolution
    x.write_data(0x03);
    x.write_data(0xFF);
    x.write_data(0xFF);
    x.write_command(0xB0); //LCD SPECIFICATION
    x.write_data(0x20);
    x.write_data(0x00);
    x.write_data(0x03); //Set HDP	799
    x.write_data(0x1F);
    x.write_data(0x01); //Set VDP	479
    x.write_data(0xDF);
    x.write_data(0x00);
    x.write_command(0xB4); //HSYNC
    x.write_data(0x03); //Set HT	928
    x.write_data(0xA0);
    x.write_data(0x00); //Set HPS	46
    x.write_data(0x2E);
    x.write_data(0x30); //Set HPW	48
    x.write_data(0x00); //Set LPS	15
    x.write_data(0x0F);
    x.write_data(0x00);
    x.write_command(0xB6); //VSYNC
    x.write_data(0x02); //Set VT	525
    x.write_data(0x0D);
    x.write_data(0x00); //Set VPS	16
    x.write_data(0x10);
    x.write_data(0x10); //Set VPW	16
    x.write_data(0x00); //Set FPS	8
    x.write_data(0x08);
    x.write_command(0xBA);
    x.write_data(0x0F); //GPIO[3:0] out 1
    x.write_command(0xB8);
    x.write_data(0x07); //GPIO3=input, GPIO[2:0]=output
    x.write_data(0x01); //GPIO0 normal
    x.write_command(0x36); //rotation
    x.write_data(0x22);
    x.write_command(0xF0); //pixel data interface
    x.write_data(0x03);
    x.delay(1);
    x.write_command(0xB8);
    x.write_data(0x0f); //GPIO is controlled by host GPIO[3:0]=output   GPIO[0]=1  LCD ON  GPIO[0]=1  LCD OFF
    x.write_data(0x01); //GPIO0 normal
    x.write_command(0xBA);
    x.write_data(0x01); //GPIO[0] out 1 --- LCD display on/off control PIN
                        // setXY(0, 0, 799, 479);
    x.write_command(0x29); //display on
    x.write_command(0xBE); //set PWM for B/L
    x.write_data(0x06);
    x.write_data(0xf0);
    x.write_data(0x01);
    x.write_data(0xf0);
    x.write_data(0x00);
    x.write_data(0x00);
    x.write_command(0xd0);
    x.write_data(0x0d);

    // x.write_command(0xE2);		//PLL multiplier, set PLL clock to 120M
    // x.write_data(0x23);	    //N=0x36 for 6.5M, 0x23 for 10M crystal
    // x.write_data(0x02);
    // x.write_data(0x04);
    // x.write_command(0xE0);		// PLL enable
    // x.write_data(0x01);
    // x.delay(10);
    // x.write_command(0xE0);
    // x.write_data(0x03);
    // x.delay(10);
    // x.write_command(0x01);		// software reset
    // x.delay(100);
    // x.write_command(0xE6);		//PLL setting for PCLK, depends on resolution
    // x.write_data(0x04);
    // x.write_data(0x93);
    // x.write_data(0xE0);
    // x.write_command(0xB0);		//LCD SPECIFICATION
    // x.write_data(0x00);	// 0x24
    // x.write_data(0x00);
    // x.write_data(0x03);		//Set HDP	799
    // x.write_data(0x1F);
    // x.write_data(0x01);		//Set VDP	479
    // x.write_data(0xDF);
    // x.write_data(0x00);
    // x.write_command(0xB4);		//HSYNC
    // x.write_data(0x03);		//Set HT	928
    // x.write_data(0xA0);
    // x.write_data(0x00);		//Set HPS	46
    // x.write_data(0x2E);
    // x.write_data(0x30);		//Set HPW	48
    // x.write_data(0x00);		//Set LPS	15
    // x.write_data(0x0F);
    // x.write_data(0x00);
    // x.write_command(0xB6);		//VSYNC
    // x.write_data(0x02);		//Set VT	525
    // x.write_data(0x0D);
    // x.write_data(0x00);		//Set VPS	16
    // x.write_data(0x10);
    // x.write_data(0x10);		//Set VPW	16
    // x.write_data(0x00);		//Set FPS	8
    // x.write_data(0x08);
    // x.write_command(0xBA);
    // x.write_data(0x05);		//GPIO[3:0] out 1
    // x.write_command(0xB8);
    // x.write_data(0x07);	    //GPIO3=input, GPIO[2:0]=output
    // x.write_data(0x01);		//GPIO0 normal
    // x.write_command(0x36);		//rotation
    // x.write_data(0x22);		// -- Set to 0x21 to rotate 180 degrees
    // x.write_command(0xF0);		//pixel data interface
    // x.write_data(0x03);
    // x.delay(10);
    // // setXY(0, 0, 799, 479);
    // x.write_command(0x29);		//display on
    // x.write_command(0xBE);		//set PWM for B/L
    // x.write_data(0x06);
    // x.write_data(0xF0);
    // x.write_data(0x01);
    // x.write_data(0xF0);
    // x.write_data(0x00);
    // x.write_data(0x00);
    // x.write_command(0xD0);
    // x.write_data(0x0D);
    // x.write_command(0x2C);

    // x.init();
    // hprintln!("{:08b}", a).unwrap();
    x.clear_screen(0);
    // x.clear_screen(0b0000000000000000);
    loop {}
}
