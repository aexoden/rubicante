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
}

impl Config {
    pub fn get_base_window_size(&self) -> (usize, usize) {
        (256, 224)
    }

    pub fn get_window_size(&self) -> (usize, usize) {
        let (base_width, base_height) = self.get_base_window_size();

        if self.incorrect_aspect {
            ((base_width as f32 * self.scale).round() as usize, (base_height as f32 * self.scale).round() as usize)
        } else {
            ((base_width as f32 * self.scale * 7.0 / 6.0).round() as usize, (base_height as f32 * self.scale) as usize)
        }
    }
}
