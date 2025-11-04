use clap::builder::styling;


pub const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Magenta.on_default().bold().italic())
    .usage(styling::AnsiColor::BrightMagenta.on_default().bold())
    .literal(styling::AnsiColor::Green.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());
