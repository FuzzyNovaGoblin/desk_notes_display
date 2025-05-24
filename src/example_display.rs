use core::convert::TryInto;
use embedded_graphics::{
    pixelcolor::{Gray8, GrayColor},
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use esp_hal::{spi::master::{ SpiDma, SpiDmaBus}};
use smoltcp::iface;

/// SPI communication error
#[derive(Debug)]
struct CommError;

pub struct ExampleDisplay<'a> {
    /// The framebuffer with one `u8` value per pixel.
    framebuffer: [u8; 64 * 63],

    /// The interface to the display controller.
    iface: SpiDmaBus<'a, esp_hal::Async>,
}

impl<'a> ExampleDisplay<'a> {
    pub fn new(framebuffer: [u8; 64 * 63], iface: SpiDmaBus<'a, esp_hal::Async>) -> Self {
        Self { framebuffer, iface }
    }

    /// Updates the display from the framebuffer.
    pub fn flush(&mut self) -> Result<(), esp_hal::spi::Error> {

        // self.iface.send_bytes(&self.framebuffer)
        let mut buffer = [0; 8];
        esp_hal::spi::master::SpiDmaBus::<'_, esp_hal::Async>::transfer(&mut self.iface, &mut buffer, &self.framebuffer)
        // embedded_hal_async::spi::SpiBus::transfer(&mut self.iface, &mut buffer, &self.framebuffer)
        //     .await
        //     .unwrap();
    }
}

impl DrawTarget for ExampleDisplay<'_> {
    type Color = Gray8;
    // `ExampleDisplay<'a>` uses a framebuffer and doesn't need to communicate with the display
    // controller to draw pixel, which means that drawing operations can never fail. To reflect
    // this the type `Infallible` was chosen as the `Error` type.
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Check if the pixel coordinates are out of bounds (negative or greater than
            // (63,63)). `DrawTarget` implementation are required to discard any out of bounds
            // pixels without returning an error or causing a panic.
            if let Ok((x @ 0..=63, y @ 0..=62)) = coord.try_into() {
                // Calculate the index in the framebuffer.
                let index: u32 = x + y * 63;
                self.framebuffer[index as usize] = color.luma();
            }
        }

        Ok(())
    }
}

impl OriginDimensions for ExampleDisplay<'_> {
    fn size(&self) -> Size {
        Size::new(64, 63)
    }
}

// let mut display = ExampleDisplay<'a> {
//     framebuffer: [0; 4096],
//     iface: SPI1,
// };

// // Draw a circle with top-left at `(22, 22)` with a diameter of `20` and a white stroke
// let circle = Circle::new(Point::new(22, 22), 20)
//     .into_styled(PrimitiveStyle::with_stroke(Gray8::WHITE, 1));

// circle.draw(&mut display)?;

// // Update the display
// display.flush().unwrap();