use image::{DynamicImage, GenericImageView, Pixel, Rgb, Rgba, RgbaImage};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

type BoxError = Box<dyn std::error::Error>;

#[derive(Debug)]
struct Color(Rgb<u8>);

impl FromStr for Color {
    type Err = BoxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rem) = s.strip_prefix("#") {
            let mut rem = rem;

            let mut parse_component = || {
                rem.get(..2)
                    .and_then(|s| {
                        rem = &rem[2..];
                        u8::from_str_radix(s, 16).ok()
                    })
                    .ok_or_else(|| "Invalid format for color, expected #rrggbb")
            };

            let r = parse_component()?;
            let g = parse_component()?;
            let b = parse_component()?;

            if rem.is_empty() {
                Ok(Color(Rgb([r, g, b])))
            } else {
                Err("Invalid format for color, expected #rrggbb".into())
            }
        } else {
            Err("Invalid format for color, expected #rrggbb".into())
        }
    }
}

/// Create images that look different in Discord light and dark themes.
#[derive(StructOpt, Debug)]
struct Options {
    /// The background color of dark mode.
    #[structopt(short, long, default_value = "#36393f", value_name = "#rrggbb")]
    dark_background: Color,

    /// The background color of light mode.
    #[structopt(short, long, default_value = "#ffffff", value_name = "#rrggbb")]
    light_background: Color,

    /// The image to be displayed in dark mode.
    #[structopt(name = "DARK_FILE")]
    dark_file: PathBuf,

    /// The image to be displayed in light mode.
    #[structopt(name = "LIGHT_FILE")]
    light_file: PathBuf,

    /// Where to write the output.
    #[structopt(name = "OUT_FILE")]
    output_file: PathBuf,
}

fn midpoint(image: &DynamicImage) -> f32 {
    let pixel_count = image.width() * image.height();
    let sum: f32 = image
        .pixels()
        .map(|(_, _, pixel)| pixel.to_luma().channels()[0] as f32 / 255.0)
        .sum();
    sum / (pixel_count as f32)
}

fn is_above<P>(pixel: P, midpoint: f32) -> bool
where
    P: Pixel<Subpixel = u8>,
{
    pixel.to_luma().channels()[0] as f32 / 255.0 > midpoint
}

fn convert(
    dark_image: &DynamicImage,
    light_image: &DynamicImage,
    dark_color: Rgb<u8>,
    light_color: Rgb<u8>,
) -> Result<RgbaImage, BoxError> {
    if dark_image.dimensions() != light_image.dimensions() {
        return Err("dark and light image dimensions must be the same".into());
    }

    let dark_dark = Rgba([
        dark_color.channels()[0],
        dark_color.channels()[1],
        dark_color.channels()[2],
        127,
    ]);
    let light_light = Rgba([
        light_color.channels()[0],
        light_color.channels()[1],
        light_color.channels()[2],
        127,
    ]);
    let dark_light = Rgba([0, 0, 0, 0]);
    let light_dark = Rgba([
        ((dark_color.channels()[0] as u16 + light_color.channels()[0] as u16) / 2) as u8,
        ((dark_color.channels()[1] as u16 + light_color.channels()[1] as u16) / 2) as u8,
        ((dark_color.channels()[2] as u16 + light_color.channels()[2] as u16) / 2) as u8,
        255,
    ]);

    let dark_midpoint = midpoint(dark_image);
    let light_midpoint = midpoint(light_image);

    let mut output = RgbaImage::new(dark_image.width(), dark_image.height());
    for y in 0..output.height() {
        for x in 0..output.width() {
            let pixel = match (
                is_above(dark_image.get_pixel(x, y), dark_midpoint),
                is_above(light_image.get_pixel(x, y), light_midpoint),
            ) {
                (false, false) => dark_dark,
                (true, true) => light_light,
                (false, true) => dark_light,
                (true, false) => light_dark,
            };
            *output.get_pixel_mut(x, y) = pixel;
        }
    }
    Ok(output)
}

fn main() -> Result<(), BoxError> {
    let options = Options::from_args();

    let dark_image = image::open(&options.dark_file)?;
    let light_image = image::open(&options.light_file)?;
    let output = convert(
        &dark_image,
        &light_image,
        options.dark_background.0,
        options.light_background.0,
    )?;
    output.save(&options.output_file)?;
    Ok(())
}
