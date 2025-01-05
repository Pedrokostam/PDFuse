// mod file_finder;
// mod loader;
// pub mod logger;
// pub mod parser;
// mod pdf;
// mod progress;

// use dashmap::DashMap;
// use log::{debug, info, LevelFilter};
// use logger::CONSOLE_LOGGER;
// use rust_i18n::t;
// use std::{
//     collections::HashMap,
//     default,
//     env::temp_dir,
//     ffi::OsStr,
//     path::Path,
//     process::Command,
//     sync::{Arc, RwLock},
// };

// use clap::Parser;
// use pdfuse_sizing::{CustomSize, Length, Size};
// use file_finder::{FileFindParams, IndexedSourcePath, SourcePath};
// use lopdf::Document;
// use parser::Args;
// pub use pdf::merge_pdfs;
// use pdf::{image_pdf::ImagePdf, Pdf};
// use progress::BusyIndicator;
// use rayon::prelude::*;

// // use rust_i18n::{i18n, set_locale, t};
// // #[macro_use]
// // extern crate rust_i18n;
// rust_i18n::i18n!(fallback = "en");

// // pub fn main() {
// //     log::set_logger(&CONSOLE_LOGGER);
// //     log::set_max_level(LevelFilter::Info);
// //     #[cfg(debug_assertions)]
// //     log::set_max_level(LevelFilter::Debug);
// //     let args = parser::Args::parse();
// //     let params = FileFindParams {
// //         max_depth: args.recursion_limit,
// //         supports_office: false,
// //         callback: Some(BusyIndicator::new()),
// //     };
// //     let source_files = file_finder::get_files(&args.files, params);
// //     let page_size = get_page_size(&args, &source_files);
// //     let documents = gather_documents(source_files, page_size, args);
// //     merge_pdfs(documents, "turbo_merge.pdf");
// // }

// pub fn get_files(args: &Args) -> Vec<IndexedSourcePath> {
//     let params = FileFindParams {
//         max_depth: args.recursion_limit,
//         supports_office: false,
//         callback: Some(BusyIndicator::new()),
//     };
//     file_finder::get_files(&args.files, params)
// }
// fn preload_libredocuments(
//     libre_path: Option<&str>,
//     indexed_sources: &[IndexedSourcePath],
// ) -> Arc<DashMap<usize, Pdf>> {
//     let Some(exe_path) = libre_path else {
//         return Arc::new(DashMap::new());
//     };
//     let binding = std::env::temp_dir().to_owned();
//     let temp_dir = binding.as_path();
//     let mut preloaded: Arc<DashMap<usize, Pdf>> = Arc::new(DashMap::new());
//     indexed_sources
//         .par_iter()
//         .filter(|is: &&IndexedSourcePath| matches!(is.source, SourcePath::LibreDocument(..)))
//         .for_each(|isd| {
//             let SourcePath::LibreDocument(doc_path) = &isd.source else {
//                 return;
//             };
//             let doc = convert_document_to_pdf(doc_path, exe_path, temp_dir);
//             preloaded.insert(isd.index, doc.into());
//         });
//     preloaded
// }

// fn convert_document_to_pdf(doc_path: &str, exe_path: &str, temp_dir: &Path) -> Document {
//     let extension_path = Path::new(doc_path).with_extension("pdf");
//     let name = extension_path
//         .file_name()
//         .expect("Changing extension to pdf should nt fail");

//     let temp_path = temp_dir.with_file_name(name);
//     let cmd = Command::new(exe_path)
//         .arg("--convert-to")
//         .arg("pdf")
//         .arg(doc_path)
//         .arg("--outdir")
//         .arg(temp_path);
//     todo!();
//     // match cmd.output() {
//     //     Ok(output) => todo!(),
//     //     Err(_) => error_t!("error.docpdf_conversion", document = doc_path),
//     // }
// }

// /// Goes through all `source_files` converting them to `Pdf`. If multiple images are in a row, they are added as one `Pdf`
// pub fn gather_documents(
//     source_files: Vec<IndexedSourcePath>,
//     page_size: CustomSize,
//     args: Args,
//     mut preloaded: Option<HashMap<usize, Pdf>>,
// ) -> Vec<Pdf> {
//     let page_size = get_page_size(&args, &source_files);

//     let mut loaded = preloaded.unwrap_or_default();
//     let mut documents: Vec<Pdf> = vec![];
//     let mut imager: Option<ImagePdf> = None;
//     for sf in source_files.iter() {
//         // if the current element was already loaded, push it and continue
//         if let Some(preloaded_pdf) = loaded.remove(&sf.index) {
//             if !matches!(&sf.source, SourcePath::Image(..)) {
//                 add_imager_if_any(&mut imager, &mut documents);
//             }
//             debug_t!(
//                 "using-preloaded-source",
//                 index = &sf.index,
//                 path = &sf.source
//             );
//             documents.push(preloaded_pdf);
//             continue;
//         }
//         // if not loaded - load it here and push.
//         match &sf.source {
//             SourcePath::Image(path) => {
//                 let image_pdf = imager.get_or_insert_with(|| {
//                     ImagePdf::new("Image", page_size, args.dpi, Some(args.margin))
//                 });
//                 image_pdf.add_image(path);
//             }
//             SourcePath::Pdf(path) => {
//                 add_imager_if_any(&mut imager, &mut documents);
//                 if let Ok(doc) = Document::load(path) {
//                     documents.push(doc.into());
//                 }
//             }
//             SourcePath::LibreDocument(_) => {
//                 panic!("LibreDocuments should already be taken care of")
//             }
//         };
//     }
//     add_imager_if_any(&mut imager, &mut documents);
//     documents
// }

// pub fn get_page_size(args: &Args, source_files: &Vec<IndexedSourcePath>) -> CustomSize {
//     let mut page_size = args.image_page_fallback_size.to_custom_size();
//     if !source_files
//         .iter()
//         .any(|x| matches!(x.source, SourcePath::Image(..)))
//     {
//         // no images, so no need to check for fallback size
//         return page_size;
//     }
//     if !args.force_image_page_fallback_size {
//         for indexed_source in source_files {
//             match &indexed_source.source {
//                 SourcePath::Pdf(p) => {
//                     if let Ok(doc) = Document::load(p) {
//                         let Some(page) = doc.page_iter().next() else {
//                             continue;
//                         };
//                         let Ok(media_box_array) = doc
//                             .get_object(page)
//                             .and_then(|p| p.as_dict())
//                             .and_then(|d| d.get(b"MediaBox"))
//                             .and_then(|mb| mb.as_array())
//                         else {
//                             debug_t!("debug.invalid_mediabox", document = p);
//                             continue;
//                         };
//                         // all sizes in points
//                         let x_min = media_box_array[0].as_float().unwrap_or_default();
//                         let y_min = media_box_array[1].as_float().unwrap_or_default();
//                         let x_max = media_box_array[2].as_float().unwrap_or_default();
//                         let y_max = media_box_array[3].as_float().unwrap_or_default();
//                         let horizontal = Length::from_points(x_max - x_min);
//                         let vertical = Length::from_points(y_max - y_min);
//                         if horizontal <= Length::zero() || vertical <= Length::zero() {
//                             debug_t!("debug.zero_mediabox", document = p);
//                             continue;
//                         }
//                         page_size = CustomSize {
//                             horizontal,
//                             vertical,
//                         };
//                         break;
//                     }
//                 }
//                 _ => continue,
//             }
//         }
//     }
//     page_size
// }

// /// if there was Some element in `imager`, finishes it and adds it to `documents`, setting `imager` to `None`
// fn add_imager_if_any(imager: &mut Option<ImagePdf>, documents: &mut Vec<Pdf>) {
//     if let Some(image_pdf) = imager.take() {
//         debug_t!(
//             "debug.adding_image_pdf",
//             page_count = image_pdf.page_count()
//         );
//         documents.push(image_pdf.into_document().into());
//     }
// }

// /// Logs translated text (with optional arguments) as info
// #[macro_export]
// macro_rules! info_t {

//     ($key:expr) => {{
//         let translated_message = rust_i18n::t!($key);
//         log::info!("{}", translated_message);
//     }};

//     ($key:expr, $($t_args:tt)+) => {{
//         let translated_message = rust_i18n::t!($key, $($t_args)*);
//         log::info!("{}", translated_message);
//     }};
// }

// /// Logs translated text (with optional arguments) as debug
// #[macro_export]
// macro_rules! debug_t {

//     ($key:expr) => {{
//         let translated_message = rust_i18n::t!($key);
//         log::debug!("{}", translated_message);
//     }};

//     ($key:expr, $($t_args:tt)+) => {{
//         let translated_message = rust_i18n::t!($key, $($t_args)*);
//         log::debug!("{}", translated_message);
//     }};
// }
// /// Logs translated text (with optional arguments) as error

// /// Logs translated text (with optional arguments) as trace
// #[macro_export]
// macro_rules! trace_t {

//     ($key:expr) => {
//         let translated_message = rust_i18n::t!($key);
//         log::trace!("{}", translated_message);
//     };

//     ($key:expr, $($t_args:tt)+) => {
//         let translated_message = rust_i18n::t!($key, $($t_args)*);
//         log::trace!("{}", translated_message);
//     };
// }

// #[macro_export]
// macro_rules! error_t {

//     ($key:expr) => {
//         let translated_message = rust_i18n::t!($key);
//         log::error!("{}", translated_message);
//     };

//     ($key:expr, $($t_args:tt)+) => {
//         let translated_message = rust_i18n::t!($key, $($t_args)*);
//         log::error!("{}", translated_message);
//     };
// }
