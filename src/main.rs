use discriminord::{Color, Error};
use std::path::PathBuf;
use structopt::StructOpt;

/// Create images that look different in Discord light and dark themes.
#[derive(StructOpt)]
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

fn main() -> Result<(), Error> {
    let options = Options::from_args();

    let dark_image = image::open(&options.dark_file)?;
    let light_image = image::open(&options.light_file)?;
    let output = discriminord::convert(
        &dark_image,
        &light_image,
        options.dark_background.0,
        options.light_background.0,
    )?;
    output.save(&options.output_file)?;
    Ok(())
}
