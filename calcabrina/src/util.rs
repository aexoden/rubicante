use fern::colors;

pub struct Position {
    pub x: u8,
    pub y: u8,
}

pub fn setup_logger() -> Result<(), fern::InitError> {
    let colors = colors::ColoredLevelConfig::default()
        .info(colors::Color::Green)
        .debug(colors::Color::BrightMagenta)
        .trace(colors::Color::BrightBlue);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{:<5}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("ff4", log::LevelFilter::Trace)
        .level_for("calcabrina", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
