use pdfuse_parameters::error::ConfigError;
use pdfuse_utils::*;

rust_i18n::i18n!();

fn main() {
    let _ = main_impl().map_err(|e| log::error!("{e}"));
    let _ = finish_progress_bar().map_err(|e| log::error!("{e}"));
}

fn set_logger() {
    pdfuse_utils::init_logger();
    let lvl = if cfg!(debug_assertions) {
        log::LevelFilter::Warn
    } else {
        log::LevelFilter::Trace
    };
    log::set_max_level(lvl);
}

fn main_impl() -> Result<(), ConfigError> {
    set_logger();

    let start_time_parse = std::time::Instant::now();

    let args = pdfuse_commandline::get_args()?;
    let params_with_paths = args.to_parameters();

    let end_time_parse: std::time::Instant = std::time::Instant::now();
    let millis_parse = (end_time_parse - start_time_parse).as_millis();
    debug_t!("debug.command_parsed_time", millis = millis_parse);
    // return Ok(());
    let start_time_merge = std::time::Instant::now();

    let (files, parameters) = params_with_paths.deconstruct();
    pdfuse_merging::load(files, &parameters);

    let end_time_merge = std::time::Instant::now();
    let millis_merge = (end_time_merge - start_time_merge).as_secs_f64();
    info_t!("time_taken", duration_seconds = millis_merge);

    Ok(())
}
