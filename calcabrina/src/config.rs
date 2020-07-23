use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Jason Lynch <jason@calindora.com>")]
pub struct Config {
    /// Final Fantasy IV ROM image from which to read data.
    pub filename: String,

    /// Scaling factor to apply to the window.
    #[clap(short, long, default_value = "3.0")]
    pub scale: f32,

    /// Display with the incorrect 8:7 aspect ratio (as opposed to 4:3).
    #[clap(short, long)]
    pub incorrect_aspect: bool,

    /// Experimental support for 16:9 output. (Or approximately 3:2 if using the incorrect aspect ratio.)
    #[clap(short, long)]
    pub widescreen: bool,
}

impl Config {
    pub fn get_base_window_size(&self) -> (f32, f32) {
        if self.widescreen {
            (256.0 * 15.0 * 4.0 / 14.0 / 3.0, 224.0 * 15.0 / 14.0)
        } else {
            (256.0 * 15.0 / 14.0, 224.0 * 15.0 / 14.0)
        }
    }

    pub fn get_window_size(&self) -> (usize, usize) {
        let (base_width, base_height) = self.get_base_window_size();

        if self.incorrect_aspect {
            (
                (base_width * self.scale).round() as usize,
                (base_height * self.scale).round() as usize,
            )
        } else {
            (
                (base_width * self.scale * 7.0 / 6.0).round() as usize,
                (base_height * self.scale) as usize,
            )
        }
    }
}
