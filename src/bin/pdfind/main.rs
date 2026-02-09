mod bundler;
mod displayable_range;
mod page_info_row;
use clap::{Arg, ArgAction, Command, value_parser};
use pdfuse_commandline::styling::TABLE_STYLE;
use pdfuse_merging::{data::LoadedDocument, error::PageSizeParseError};
use pdfuse_parameters::path::SafePath;
use pdfuse_sizing::{Unit, paper::Page};
use tabled::{
    Table,
    settings::{Alignment, object::Columns},
};

use crate::page_info_row::PageInfoRow;

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
    let bundle_arg = Arg::new("no_bundle")
        .long("no-bundle")
        .help("When listing page sizes, list every single page explicitly. Do not bundle them.")
        .action(ArgAction::SetTrue);
    let cmd = Command::new("PDFind")
        .author("Maciej Krosta")
        .color(clap::ColorChoice::Auto)
        .arg_required_else_help(true)
        .styles(pdfuse_commandline::styling::STYLES)
        .arg(file_arg)
        .arg(bundle_arg)
        .arg(unit_arg);
    let matches = cmd.get_matches();
    let files_to_check: Vec<SafePath> = matches
        .get_many::<SafePath>("files")
        .expect("Files are required arguments")
        .cloned()
        .collect();
    let unit = matches.get_one::<Unit>("unit").cloned();
    let bundle = !matches.get_flag("no_bundle");
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
        let doc = load_result.expect("Doc should already be loaded");
        if doc.page_count() == 0 {
            println!("\x1b[33mDocument has no pages\x1b[0m");
            continue;
        }
        // println!("{} pages", doc.page_count());

        let sizes: Vec<Result<Page, PageSizeParseError>> = doc
            .page_sizes()
            .into_iter()
            .map(|x| x.map(Page::normalize))
            .collect();
        let table_data: Vec<PageInfoRow> = sizes
            .into_iter()
            .enumerate()
            .map(|r| PageInfoRow::new(r.0 + 1, r.1.map_err(|e| e.into()) ))
            .collect();

        handle_data(table_data, bundle,unit);
    }
}
fn handle_data(table_data: Vec<PageInfoRow>, bundle: bool, unit:Option<Unit>) {
    let mut tabelka: Table;
    if table_data.is_empty() {
        return;
    }
    if !bundle {
        tabelka = PageInfoRow::build_table(table_data, unit);
    } else {
        let bundled = PageInfoRow::bundle_data(table_data);
        tabelka = PageInfoRow::build_table(bundled, unit);
    }
    tabelka
        .with(TABLE_STYLE)
        .modify(Columns::new(0..), Alignment::center());
    tabelka.to_string();
    println!("{tabelka}");
}
