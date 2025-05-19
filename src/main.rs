use pdfuse_utils::info_t;
rust_i18n::i18n!();

fn main() {
    log::set_logger(&pdfuse_utils::CONSOLE_LOGGER).expect("Setting logger cannot fail!");
    #[cfg(debug_assertions)]
    log::set_max_level(log::LevelFilter::Trace);
    #[cfg(not(debug_assertions))]
    log::set_max_level(log::LevelFilter::Trace);
    let start_time = std::time::Instant::now();
    let parameters = match pdfuse_parameters::ParametersWithPaths::parse() {
        Ok(p) => p,
        Err(e) => {
            print!("{e}");
            std::process::exit(1);
        }
    };
    pdfuse_merging::load(parameters.files.to_owned(), &parameters.parameters);
    let end_time = std::time::Instant::now();
    info_t!("time_taken",duration_seconds=(end_time-start_time).as_secs_f32()); 
}
