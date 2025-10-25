use pdfuse_parameters::error::ConfigError;
use pdfuse_parameters::path::SourcePath;
use pdfuse_parameters::ParametersWithPaths;
use pdfuse_utils::*;

rust_i18n::i18n!();

fn main() {
    let _ = main_impl().map_err(|e| log::error!("{e}"));
    let _ = finish_progress_bar().map_err(|e| log::error!("{e}"));
}
fn main_impl() -> Result<(), ConfigError> {
    pdfuse_utils::init_logger();
    log::set_max_level(log::LevelFilter::Trace);

    let start_time_parse = std::time::Instant::now();

    let args = pdfuse_commandline::get_args()?;
    let params_with_paths = args.to_parameters();

    let end_time_parse: std::time::Instant = std::time::Instant::now();
    let millis_parse = (end_time_parse - start_time_parse).as_millis();
    debug_t!("debug.command_parsed_time", millis = millis_parse);
    return Ok(());
    let start_time_merge = std::time::Instant::now();

    let (files, parameters) = params_with_paths.deconstruct();
    pdfuse_merging::load(files, &parameters);

    let end_time_merge = std::time::Instant::now();
    let millis_merge = (end_time_merge - start_time_merge).as_millis();
    info_t!("time_taken", duration_seconds = millis_merge);
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
