use color_print::cstr;
use indoc::indoc;

pub const ABOUT: &str =
    "Command-line tool to concatenate images, documents and PDFs into a single PDF file.";

pub const AFTER_HELP: &str = cstr!("Specify  <s><i>--help</></> to get detailed help and information about specifying sizes and reusing configurations.");
pub const AFTER_LONG_HELP: &str = cstr!(
    r#"<bright-cyan><s><i>Reusing configuration</></></>

It's possible to save common settings to simplify command-line usage.

By default, the app loads "config.toml" from the executable's directory. If missing, defaults are used.

Use <s><i>--config</></> to load a different existing file. It can define any parameters; missing ones use defaults. Command-line options override config values.

Example:
─────config.toml─────┐
log = "debug"        │
recursion_limit = 10 │
─────────────────────┘
Command-line: --recursion_limit 5 --lossless  
Final config: log = debug, recursion_limit = 5, lossless = true

Use <s><i>--save-config</></> [filepath] to save current config for reuse.

Ignored in config files:
 • <s><i>--save-config</></>
 • <s><i>--config</></>
 • <s><i>--whatif</></> (good for creating new configs!)
 • <s><i>files</></>

<s><i>TIP</></> For drag & drop, use a shortcut with <s><i>--config</></> pointing to a preset (e.g., output to Desktop).

<bright-cyan><s><i>Sizes</></></>

Sizes can be:

1) Dimensions (width by height) with units:
   • Dimensions must have units and be separated aby at least one of: 'x', '-', ' '.
   • If height omitted, it's set equal to width.
   • Units (can mix):
     • Meters <s><i>m</></>, Millimeters <s><i>mm</></>, Centimeters <s><i>cm</></>, Inches <s><i>in</></>, Points <s><i>pt</></>
   • Examples: 5mm x 7mm, 1in, 5mm-7mm, 5mm 7mm

2) ISO sheets (A, B, C series, ranks 0–13):
   • Examples: A4, B5, C11"#
);

pub const ALPHABETIC_FILE_SORTING_HELP: &str = "Sort input paths alphabetically.";
pub const ALPHABETIC_FILE_SORTING_LONG_HELP: &str = r#"Sort input paths alphabetically.

By default, files are processed in the order given.
Enabling this option will sort all found files by path after collection.
Useful when using drag & drop, as input order may be unpredictable."#;

pub const CONFIG_HELP: &str = "Load a configuration file with preset options.";

pub const CONFIRM_EXIT_HELP: &str = "Wait for user input before closing the application.";

pub const DPI_HELP: &str = "DPI (dots per inch) to use when processing images.";

pub const FILES_HELP: &str = "Directories and files to be processed.";
pub const FILES_LONG_HELP: &str = cstr!(
    r#"Directories and files to be processed.

Both directory paths and file paths are acceptable.
If a directory is provided PDFuse will look for applicable file recursively until depth of <s><i>--recursion-limit</></> is reached."#
);

pub const FORCE_IMAGE_PAGE_FALLBACK_SIZE_HELP: &str =
    "Always use the fallback page size for images, ignoring any prior document size.";

pub const IMAGE_PAGE_FALLBACK_SIZE_HELP: &str = "Fallback page size for images.";
pub const IMAGE_PAGE_FALLBACK_SIZE_LONG_HELP: &str = cstr!(
    r#"Fallback page size for images.

If an image follows a document, it uses the previous page size.
If no page size is known, this fallback size is used.

See the <bright-cyan><s><i>Sizes</></></> section for supported formats."#
);

pub const LANGUAGE_HELP: &str = "Specify a language file identifier.";

pub const LIBREOFFICE_PATH_HELP: &str = "Paths to LibreOffice executables used for converting documents.";
pub const LIBREOFFICE_PATH_LONG_HELP: &str = r#"Paths to LibreOffice executables used for converting documents.

If not found, LibreOffice-based conversions will be disabled."#;

pub const LOG_HELP: &str = "Controls which messages are logged into console.";
pub const LOG_LONG_HELP: &str = cstr!(r#"Controls which messages are logged into console.

If you just want to disable logging completely, you can also use <s><i>--quiet</></> (<s><i>-q</></>)"#);

pub const LOSSLESS_HELP: &str = "Use only lossless compression for images.";
pub const LOSSLESS_LONG_HELP: &str = cstr!(
    r#"Use only lossless compression for images.

<s><i><y>Warning!</></></> This will dramatically increase the size of the output file!

To explicitly disable lossless compression use <s><i>--no-lossless</></>"#
);

pub const MARGIN_HELP: &str = "Margin to apply around images.";

pub const OUTPUT_DIRECTORY_HELP: &str = "Directory where the output file will be saved.";
pub const OUTPUT_DIRECTORY_LONG_HELP: &str = r#"Directory where the output file will be saved.

A unique filename will be generated using the current date and time and the selected language."#;

pub const OUTPUT_FILE_HELP: &str = "Path to the final output file.";
pub const OUTPUT_FILE_LONG_HELP: &str = r#"Path for the output file.

The file will overwrite any existing file at the target path."#;

pub const QUALITY_HELP: &str = "JPEG compression quality (1-100).";
pub const QUALITY_LONG_HELP: &str = r#"JPEG compression quality (1-100).

Ignored when <s><i>--lossless</></> is applied."#;

pub const RECURSION_LIMIT_HELP: &str = "Recursion depth limit for directories.";
pub const RECURSION_LIMIT_LONG_HELP: &str = r#"Recursion depth limit for directories.

0 — only files in specified directory are scanned.
1 — files in any immediate subfolders are also scanned."#;

pub const SAVE_CONFIG_HELP: &str =
    "Save current input options to a TOML config file. Use `-` to output to stdout.";

pub const WHAT_IF_HELP: &str = "Perform a dry run without creating any PDF output.";
pub const WHAT_IF_LONG_HELP: &str = r#"Perform a dry run without creating any PDF output.

You can still save the configuration with <s><i>--save-config</></>!"#;
