use pdfuse_parameters::{ConfigError, ParametersWithPaths};
use pdfuse_utils::*;

rust_i18n::i18n!();

fn main() {
    let _ = main_impl().map_err(|e| println!("{e}"));
}
fn main_impl() -> Result<(), ConfigError> {
    log::set_logger(&pdfuse_utils::CONSOLE_LOGGER).expect("Setting logger cannot fail!");
    log::set_max_level(log::LevelFilter::Trace);
    error_t!("error.image_invalid_format");
    let start_time_parse = std::time::Instant::now();
    let args = pdfuse_parameters::get_args()?;
    let params_with_paths = ParametersWithPaths::new(args);
    let end_time_parse: std::time::Instant = std::time::Instant::now();
    debug_t!("debug.command_parsed_time",millis = (end_time_parse-start_time_parse).as_millis());

    let start_time_processing = std::time::Instant::now();
    let (files, parameters) = params_with_paths.deconstruct();
    pdfuse_merging::load(files,&parameters);
    let end_time_processing = std::time::Instant::now();
    info_t!("time_taken",duration_seconds=(end_time_processing-start_time_processing).as_secs_f32());

    
    // #[cfg(debug_assertions)]
    // log::set_max_level(log::LevelFilter::Trace);
    // #[cfg(not(debug_assertions))]
    // log::set_max_level(log::LevelFilter::Trace);
    // let start_time = std::time::Instant::now();
    // let parameters = match pdfuse_parameters::ParametersWithPaths::parse() {
    //     Ok(p) => p,
    //     Err(e) => {
    //         print!("{e}");
    //         std::process::exit(1);
    //     }
    // };
    // pdfuse_merging::load(parameters.files.to_owned(), &parameters.parameters);
    // let end_time = std::time::Instant::now();
    // info_t!("time_taken",duration_seconds=(end_time-start_time).as_secs_f32());
    Ok(())
}
