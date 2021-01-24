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

fn color_lerp(zero: Rgb<u8>, one: Rgb<u8>, luma: f32, alpha: f32) -> Rgba<u8> {
    let luma = luma.max(0.0).min(1.0);
    let alpha = alpha.max(0.0).min(1.0);

    let zero_r = zero.channels()[0] as f32;
    let zero_g = zero.channels()[1] as f32;
    let zero_b = zero.channels()[2] as f32;
    let one_r = one.channels()[0] as f32;
    let one_g = one.channels()[1] as f32;
    let one_b = one.channels()[2] as f32;

    Rgba([
        (zero_r * (1.0 - luma) + one_r * luma) as u8,
        (zero_g * (1.0 - luma) + one_g * luma) as u8,
        (zero_b * (1.0 - luma) + one_b * luma) as u8,
        (alpha * 255.0) as u8,
    ])
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

    let mut output = RgbaImage::new(dark_image.width(), dark_image.height());

    for y in 0..output.height() {
        for x in 0..output.width() {
            let dark_value = dark_image.get_pixel(x, y).to_luma().channels()[0] as f32 / 255.0;
            let light_value = light_image.get_pixel(x, y).to_luma().channels()[0] as f32 / 255.0;

            let output_alpha = (dark_value - light_value + 1.0) / 2.0;
            let output_luma = if output_alpha == 0.0 {
                0.0
            } else {
                dark_value / 2.0 / output_alpha
            };
            *output.get_pixel_mut(x, y) =
                color_lerp(dark_color, light_color, output_luma, output_alpha);
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
