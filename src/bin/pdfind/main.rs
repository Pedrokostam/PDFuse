mod displayable_range;
mod page_info_row;
use std::collections::VecDeque;

use clap::{Arg, ArgAction, Command, value_parser};
use pdfuse_commandline::styling::TABLE_STYLE;
use pdfuse_merging::{data::LoadedDocument, error::PageSizeParseError};
use pdfuse_parameters::path::SafePath;
use pdfuse_sizing::{
    Unit,
    page::Page,
};
use tabled::{
    Table,
    settings::{Alignment, object::Columns},
};

use crate::page_info_row::PageInfoRow;

struct Bundler {
    pub pages: Vec<usize>,
    pub text: String,
    pub value: PageInfoRow,
}
impl Bundler {
    pub fn convert(self) -> PageInfoRow {
        PageInfoRow {
            pages: self.pages.into(),
            size: self.value.size,
            standard: self.value.standard,
            unit: self.value.unit,
            error: self.value.error,
        }
    }
    pub fn new(piw: PageInfoRow) -> Self {
        Bundler {
            pages: piw.pages.collect(),
            text: piw.size_or_error(),
            value: piw,
        }
    }
}

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
        .unwrap()
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
        let doc = load_result.unwrap();
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
        let table_data: VecDeque<PageInfoRow> = sizes
            .into_iter()
            .enumerate()
            .map(|r| PageInfoRow::new(r.0 + 1, r.1.map_err(|e| e.into()), unit))
            .collect();

        handle_data(table_data, bundle);

        // let mut dup_check: Vec<Page> = vec![];
        // let mut has_nones = false;
        // for ss in sizes.iter() {
        //     has_nones |= ss.is_none();
        //     if let Some(sss) = ss {
        //         if !dup_check.contains(sss) {
        //             dup_check.push(*sss);
        //         }
        //     }
        // }
        // if dup_check.is_empty() {
        //     print_size(None, None, unit);
        // } else if dup_check.len() == 1 && !has_nones {
        //     print_size(Some(&dup_check[0]), None, unit);
        // } else {
        //     let max_width = sizes
        //         .iter()
        //         .flatten()
        //         .map(|x| x.to_custom_size().as_unit_string(unit).len())
        //         .max();
        //     for (index, size) in sizes.iter().enumerate() {
        //         print!("   Page {}: ", index + 1);
        //         print_size(size.as_ref(), max_width, unit);
        //     }
        // }
    }
}
fn handle_data(mut table_data: VecDeque<PageInfoRow>, bundle: bool) {
    let mut tabelka: Table;
    if table_data.is_empty() {
        return;
    }
    if !bundle {
        tabelka = Table::new(table_data);
    } else {
        let front = table_data.pop_front().unwrap();
        let mut bundler = Bundler::new(front);
        let mut bundled_data: Vec<PageInfoRow> = vec![];
        for piw in table_data.into_iter() {
            if piw.size_or_error() == bundler.text {
                bundler.pages.push(piw.pages.first())
            } else {
                bundled_data.push(bundler.convert());
                bundler = Bundler::new(piw);
            }
        }
        bundled_data.push(bundler.convert());
        tabelka = Table::new(bundled_data);
    }
    tabelka
        .with(TABLE_STYLE)
        .modify(Columns::new(0..), Alignment::center())
        .to_string();
    println!("{tabelka}");
}
