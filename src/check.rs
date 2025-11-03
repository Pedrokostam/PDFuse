use clap::{builder::styling, value_parser, Arg, ArgAction, Command};
use pdfuse_merging::data::LoadedDocument;
use pdfuse_parameters::path::SafePath;
use pdfuse_sizing::{
    page::{CustomPage, Page},
    Size, Unit,
};

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Magenta.on_default().bold().italic())
    .usage(styling::AnsiColor::BrightMagenta.on_default().bold())
    .literal(styling::AnsiColor::Green.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());
pub fn main() {
    let file_arg = Arg::new("files")
        .required(true)
        .num_args(1..)
        .value_parser(value_parser!(SafePath));
    let unit_arg = Arg::new("unit")
        .long("unit")
        .short('u')
        .hide_default_value(true)
        .default_value(Unit::default().to_string())
        .help("What unit should be used for displaying sizes. [m, mm, cm, in, pt]")
        .value_parser(Unit::try_from_string);

    let cmd = Command::new("PDFind")
        .author("Maciej Krosta")
        .color(clap::ColorChoice::Auto)
        .arg_required_else_help(true)
        .styles(STYLES)
        .arg(file_arg)
        .arg(unit_arg);
    let matches = cmd.get_matches();
    let files_to_check: Vec<SafePath> = matches
        .get_many::<SafePath>("files")
        .unwrap()
        .cloned()
        .collect();
    let unit = matches
        .get_one::<Unit>("unit")
        .unwrap_or_default()
        .to_owned();
    for file in files_to_check {
        println!("\nFile \x1b[36m{}\x1b[0m ", file);
        if !file.exists() {
            println!("\x1b[31mDoes not exist\x1b[0m");
            continue;
        }
        let load_result = LoadedDocument::load_pdf(file.as_ref());
        if let Err(e) = load_result {
            println!("\x1b[31m{}\x1b[0m", e);
            continue;
        }
        let doc = load_result.unwrap();
        if doc.page_count() == 0 {
            println!("\x1b[33mDocument has no pages\x1b[0m");
            continue;
        }
        println!("{} pages", doc.page_count());

        let sizes: Vec<Option<Page>> = doc
            .page_sizes()
            .into_iter()
            .map(|x| x.map(Page::normalize))
            .collect();
        let mut dup_check: Vec<Page> = vec![];
        let mut has_nones = false;
        for ss in sizes.iter() {
            has_nones |= ss.is_none();
            if let Some(sss) = ss {
                if !dup_check.contains(sss) {
                    dup_check.push(*sss);
                }
            }
        }
        if dup_check.is_empty() {
            print_size(None, None, unit);
        } else if dup_check.len() == 1 && !has_nones {
            print_size(Some(&dup_check[0]), None, unit);
        } else {
            let max_width = sizes
                .iter()
                .flatten()
                .map(|x| x.to_custom_size().as_unit_string(unit).len())
                .max();
            for (index, size) in sizes.iter().enumerate() {
                print!("   Page {}: ", index + 1);
                print_size(size.as_ref(), max_width, unit);
            }
        }
    }
}
fn print_size(page_size: Option<&Page>, pad: Option<usize>, unit: Unit) {
    let width = pad.unwrap_or_default();
    match page_size {
        Some(pagina) => {
            if matches!(pagina, Page::Custom(_)) {
                println!(
                    "{custom:^width$}",
                    custom = pagina.to_custom_size().as_unit_string(unit),
                    width = width
                );
            } else {
                println!(
                    "{custom:^width$} - {standard}",
                    custom = pagina.to_custom_size().as_unit_string(unit),
                    standard = pagina,
                    width = width
                );
            }
        }
        None => println!("\x1b[33mPage size cannot be determined\x1b[0m"),
    };
}
