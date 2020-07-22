use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Jason Lynch <jason@calindora.com>")]
pub struct Config {
    /// Final Fantasy IV ROM image from which to read data.
    pub filename: String,
}
