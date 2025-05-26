mod arguments;
mod commandline;
mod commandline_help;

pub use commandline::get_args;

pub use arguments::Args;
rust_i18n::i18n!();

#[cfg(test)]
mod tests {
   
}
