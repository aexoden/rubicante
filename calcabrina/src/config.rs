use clap::Clap;
use ggez_goodies::{Point2, Vector2};

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

    pub fn get_scale_vector(&self) -> Vector2 {
        let (base_window_width, base_window_height) = self.get_base_window_size();
        let (window_width, window_height) = self.get_window_size();

        Vector2::new(
            window_width as f32 / base_window_width,
            window_height as f32 / base_window_height,
        )
    }

    pub fn get_standard_offset(&self) -> Point2 {
        let (window_width, window_height) = self.get_window_size();
        let scale = self.get_scale_vector();

        let aspect = if self.incorrect_aspect {
            8.0 / 7.0
        } else {
            4.0 / 3.0
        };

        let x_offset =
            (window_width as f32 - (window_height as f32 * aspect)) / 2.0 + 8.0 * scale.x;

        Point2::new(x_offset, 8.0 * scale.y)
    }
}
