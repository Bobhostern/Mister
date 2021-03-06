use image::{Channel, Image};
use palette::Colora; // Use Colora as a generic color.
use super::{ImageFormat, ImageFormatError};
use std::fmt::{Display, Debug, Formatter, Error};
use std::error::Error as StdError;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
/// Represents the channels of an RGBA image
pub enum RgbaChannel {
    /// Red channel
    Red,
    /// Green channel
    Green,
    /// Blue channel
    Blue,
    /// Alpha channel
    Alpha
}

// got lower upper inclusive
#[derive(Debug)]
pub struct InvalidData<T: Debug>(T, T, T, bool);
impl<T: Display + Debug> Display for InvalidData<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.3 {
            write!(f, "got {}, expected value in [{}, {}]", self.0, self.1, self.2)
        } else {
            write!(f, "got {}, expected value in ({}, {})", self.0, self.1, self.2)
        }
    }
}

impl<T: Display + Debug> StdError for InvalidData<T> {
    fn description(&self) -> &str { "Invalid data" }
}

/// Stores an RGBA format image
pub struct RgbaImage {
    image: Image<f32>,
    channels: [bool; 4],
    width: usize,
    height: usize
}

macro_rules! channel {
    ($name:ident, $color:ident using $val:path) => {
        /// Return the $color channel
        pub fn $color(&self) -> &Channel<f32> {
            self.image.channel($name::to_channel(&$val)).unwrap()
        }
    };

    // NOTE For now, change the name yourself
    ($name:ident, mutable $color:ident using $val:path as $color_mut:ident) => {
        // TODO Wait for #29599 (https://github.com/rust-lang/rust/issues/29599) to land
        /// Return the $color channel mutably
        pub fn $color_mut(&mut self) -> &mut Channel<f32> {
            self.image.channel_mut($name::to_channel(&$val)).unwrap()
        }
    }
}

macro_rules! get_channel {
    ($color:ident, $cname:ident with $v:path => $name:ident) => {
        /// Extracts the channel visibility for channel $color
        pub fn $cname(&self) -> bool {
            self.channels[$name::to_channel(&$v)]
        }
    }
}

impl RgbaImage {
    /// Creates a new RgbaImage
    pub fn new(w: usize, h: usize) -> RgbaImage {
        let mut i = Image::new(w * h);
        i.create_channel(0.0);
        i.create_channel(0.0);
        i.create_channel(0.0);
        i.create_channel(1.0);
        RgbaImage {
            image: i,
            channels: [false; 4],
            width: w,
            height: h
        }
    }

    fn to_channel(c: &RgbaChannel) -> usize {
        match c {
            &RgbaChannel::Red => 0,
            &RgbaChannel::Green => 1,
            &RgbaChannel::Blue => 2,
            &RgbaChannel::Alpha => 3,
        }
    }

    get_channel!(red, is_red_visible with RgbaChannel::Red => RgbaImage);
    get_channel!(green, is_green_visible with RgbaChannel::Green => RgbaImage);
    get_channel!(blue, is_blue_visible with RgbaChannel::Blue => RgbaImage);
    get_channel!(alpha, is_alpha_visible with RgbaChannel::Alpha => RgbaImage);

    channel!(RgbaImage, red using RgbaChannel::Red);
    channel!(RgbaImage, mutable red using RgbaChannel::Red as red_mut);
    channel!(RgbaImage, green using RgbaChannel::Green);
    channel!(RgbaImage, mutable green using RgbaChannel::Green as green_mut);
    channel!(RgbaImage, blue using RgbaChannel::Blue);
    channel!(RgbaImage, mutable blue using RgbaChannel::Blue as blue_mut);
    channel!(RgbaImage, alpha using RgbaChannel::Alpha);
    channel!(RgbaImage, mutable alpha using RgbaChannel::Alpha as alpha_mut);
}

/// Errors for RGBA images
pub type RgbaImageError = ImageFormatError<RgbaChannel>;

// Our RgbaImage uses channels to store pixel information like this
// 0 ----------------> width-1
// width ------------> 2*width-1
// 2*width ----------> 3*width-1
// ... --------------> ...
// (height-1)*width -> height*width-1
impl ImageFormat<f32> for RgbaImage {
    type ChannelName = RgbaChannel;
    type ValidationError = InvalidData<f32>;

    fn channel_count(&self) -> usize { self.image.count() }
    fn set_channel_visible(&mut self, c: &RgbaChannel, enabled: bool) {
        self.channels[RgbaImage::to_channel(c)] = enabled;
    }
    fn is_channel_visible(&self, c: &RgbaChannel) -> bool {
        self.channels[RgbaImage::to_channel(c)]
    }
    fn channel(&self, c: &RgbaChannel) -> &Channel<f32> {
        self.image.channel(RgbaImage::to_channel(c)).expect("RgbaImage internal error: missing channel")
    }
    fn channel_mut(&mut self, c: &RgbaChannel) -> &mut Channel<f32> {
        self.image.channel_mut(RgbaImage::to_channel(c)).expect("RgbaImage internal error: missing channel")
    }

    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }

    fn validate(&self) -> Result<(), Self::ValidationError> {
        for i in 0..self.image.count() {
            let v = self.image.channel(i).unwrap().iter().find(|x| **x > 1.0 || **x < 0.0);
            if let Some(v) = v {
                return Err(InvalidData(*v, 0.0, 1.0, true));
            }
        }
        Ok(())
    }

    fn pixel(&self, x: usize, y: usize) -> Result<Colora, RgbaImageError> {
        if x >= self.width() || y >= self.height() {
            return Err(ImageFormatError::OutOfBounds(x, y))
        }
        let loc = y*self.width() + x;
        let r = if self.is_red_visible() {
            *self.red().get(loc).ok_or(ImageFormatError::MissingData(RgbaChannel::Red, x, y))?
        } else {
            0.0
        };
        let g = if self.is_green_visible() {
            *self.green().get(loc).ok_or(ImageFormatError::MissingData(RgbaChannel::Green, x, y))?
        } else {
            0.0
        };
        let b = if self.is_blue_visible() {
            *self.blue().get(loc).ok_or(ImageFormatError::MissingData(RgbaChannel::Blue, x, y))?
        } else {
            0.0
        };
        let a = if self.is_alpha_visible() {
            *self.alpha().get(loc).ok_or(ImageFormatError::MissingData(RgbaChannel::Alpha, x, y))?
        } else {
            1.0
        };
        Ok(Colora::rgb(r, g, b, a))
    }

    fn set_pixel(&mut self, x: usize, y: usize, c: Colora) -> Result<(), RgbaImageError> {
        use palette::Rgba;

        if x >= self.width() || y >= self.height() {
            return Err(ImageFormatError::OutOfBounds(x, y))
        }
        let loc = y*self.width() + x;
        let (r, g, b, a) = Into::<Rgba>::into(c).to_pixel();
        self.red_mut().get_mut(loc).map(|x| *x = r).ok_or(ImageFormatError::MissingData(RgbaChannel::Red, x, y))?;
        self.green_mut().get_mut(loc).map(|x| *x = g).ok_or(ImageFormatError::MissingData(RgbaChannel::Green, x, y))?;
        self.blue_mut().get_mut(loc).map(|x| *x = b).ok_or(ImageFormatError::MissingData(RgbaChannel::Blue, x, y))?;
        self.alpha_mut().get_mut(loc).map(|x| *x = a).ok_or(ImageFormatError::MissingData(RgbaChannel::Alpha, x, y))?;
        Ok(())
    }

    fn data(&self) -> Vec<Vec<f32>> {
        self.red().iter().cloned()
            .zip(self.green().iter().cloned())
            .zip(self.blue().iter().cloned())
            .zip(self.alpha().iter().cloned())
            .map(|(((r, g), b), a)| {
                vec![r, g, b, a]
            }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{RgbaImage, ImageFormat};
    use palette::Rgba;

    #[test]
    fn rgbaimage_creation() {
        let image = RgbaImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                println!("{} {}", x, y);
                let pixel = image.pixel(x, y).map::<_, _>(|x| Into::<Rgba>::into(x).to_pixel::<(f32, _, _, _)>());
                assert!(pixel.is_ok());
                assert_eq!(pixel.unwrap(), Rgba::new(0.0, 0.0, 0.0, 1.0).to_pixel())
            }
        }
    }
}
