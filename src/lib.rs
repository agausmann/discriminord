use image::{DynamicImage, GenericImageView, Pixel, Rgb, Rgba, RgbaImage};
use std::str::FromStr;

pub type Error = Box<dyn std::error::Error>;

pub struct Color(pub Rgb<u8>);

impl FromStr for Color {
    type Err = Error;

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

pub fn convert(
    dark_image: &DynamicImage,
    light_image: &DynamicImage,
    dark_color: Rgb<u8>,
    light_color: Rgb<u8>,
) -> Result<RgbaImage, Error> {
    let larger_width = dark_image.width().max(light_image.width());
    let larger_height = dark_image.height().max(light_image.height());

    let dark_start = (
        (larger_width - dark_image.width()) / 2,
        (larger_height - dark_image.height()) / 2,
    );
    let dark_end = (
        dark_start.0 + dark_image.width(),
        dark_start.1 + dark_image.height(),
    );

    let light_start = (
        (larger_width - light_image.width()) / 2,
        (larger_height - light_image.height()) / 2,
    );
    let light_end = (
        light_start.0 + light_image.width(),
        light_start.1 + light_image.height(),
    );

    let mut output = RgbaImage::new(larger_width, larger_height);

    for y in 0..output.height() {
        for x in 0..output.width() {
            let dark_value = {
                if dark_start.0 <= x && x < dark_end.0 && dark_start.1 <= y && y < dark_end.1 {
                    let (x, y) = (x - dark_start.0, y - dark_start.1);
                    dark_image.get_pixel(x, y).to_luma().channels()[0] as f32 / 255.0
                } else {
                    1.
                }
            };

            let light_value = {
                if light_start.0 <= x && x < light_end.0 && light_start.1 <= y && y < light_end.1 {
                    let (x, y) = (x - light_start.0, y - light_start.1);
                    light_image.get_pixel(x, y).to_luma().channels()[0] as f32 / 255.0
                } else {
                    1.
                }
            };

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

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::{Color, Error};
    use image::{DynamicImage, ImageFormat};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn convert(
        dark_image: &[u8],
        light_image: &[u8],
        dark_color: &str,
        light_color: &str,
    ) -> Result<Box<[u8]>, JsValue> {
        || -> Result<Box<[u8]>, Error> {
            let dark_image = image::load_from_memory(dark_image)?;
            let light_image = image::load_from_memory(light_image)?;
            let dark_color = dark_color.parse::<Color>()?.0;
            let light_color = light_color.parse::<Color>()?.0;

            let output = crate::convert(&dark_image, &light_image, dark_color, light_color)?;

            let mut output_png = Vec::new();
            DynamicImage::ImageRgba8(output).write_to(&mut output_png, ImageFormat::Png)?;
            Ok(output_png.into_boxed_slice())
        }()
        .map_err(|e| e.to_string().into())
    }
}
