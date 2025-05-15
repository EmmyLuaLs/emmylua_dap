use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;


const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");


pub fn init_stderr_logger(level: LevelFilter) {
    let logger = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S %:z"),
                record.level(),
                record.target(),
                message
            ))
        })
        // set level
        .level(level)
        // set output
        .chain(std::io::stderr());

    if let Err(e) = logger.apply() {
        eprintln!("Failed to apply logger: {:?}", e);
        return;
    }

    log::info!("{} v{}", CRATE_NAME, CRATE_VERSION);
}
