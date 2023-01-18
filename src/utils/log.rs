// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
pub fn setup_logger() -> Result<(), fern::InitError> {
    use fern::colors::{Color, ColoredLevelConfig};
    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };
    let colors = ColoredLevelConfig::new()
        .trace(Color::Cyan)
        .debug(Color::Magenta)
        .info(Color::Green)
        .warn(Color::BrightYellow)
        .error(Color::Red);

    let mut dispatch = fern::Dispatch::new().level(level);

    #[cfg(debug_assertions)]
    {
        dispatch = dispatch.chain(
            fern::Dispatch::new()
                .format(move |out, message, record| {
                    let module_path: Vec<&str> =
                        record.module_path().unwrap_or("").split("::").collect();
                    let len = module_path.len();
                    let module_path = if len > 2 && module_path[0] == "factory_management_utils" {
                        format!("..{}::{}", module_path[len - 2], module_path[len - 1])
                    } else {
                        record.module_path().unwrap_or("").to_string()
                    };
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        module_path,
                        colors.color(record.level()),
                        message
                    ))
                })
                .chain(std::io::stdout()),
        );
    }
    dispatch
        .chain(
            fern::Dispatch::new()
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{}[{}][{}] {}",
                        chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                        record.module_path().unwrap_or(""),
                        record.level(),
                        message
                    ))
                })
                .chain(fern::log_file("output.log")?),
        )
        .apply()?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn setup_logger() {}
