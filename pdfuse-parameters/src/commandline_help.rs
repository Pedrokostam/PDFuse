use color_print::cstr;
use indoc::indoc;

pub const FILES_HELP: &str = "Directories and files to be processed.";
pub const FILES_LONG_HELP: &str = cstr!(
    r#"Directories and files to be processed.

Both directory paths and file paths are acceptable.
If a directory is provided PDFuse will look for applicable file recursively until depth of <s><i>--recursion-limit</></> is reached."#
);

pub const SAVE_CONFIG_HELP: &str =
    "Save input parameters as a TOML configuration file or send to stdout with `-`.";

pub const CONFIRM_EXIT_HELP: &str = "Require user input before closing the app.";

pub const WHAT_IF_HELP: &str = "Dry run: run the program without outputting files.";

pub const LANGUAGE_HELP: &str = "Specify a language file identifier.";

pub const CONFIG_HELP: &str = "Path to a custom configuration file.";

pub const RECURSION_LIMIT_HELP: &str = "Recursion depth limit for directories.";
pub const RECURSION_LIMIT_LONG_HELP: &str = r#"Recursion depth limit for directories.

Limit of 0 mean only the files in the specified directory are scanned.
Limit of 1 means the files in any immediate subfolders are also scanned, and so on."#;

pub const IMAGE_PAGE_FALLBACK_SIZE_HELP: &str = "Fallback page size when adding images.";
pub const IMAGE_PAGE_FALLBACK_SIZE_LONG_HELP: &str = cstr!(
    r#"Fallback page size when adding images.

Images will usually use the same page size as the preceding document or PDF.
If nothing precedes an image it will be placed on page size equal to the fallback size.

Information about how to express a page size can be found in the <bright-cyan><s><i>Sizes</></></> section."#
);

pub const DPI_HELP: &str = "DPI used when adding images.";

pub const QUALITY_HELP: &str = "Quality of JPEG image compression.";
pub const QUALITY_LONG_HELP: &str = r#"Quality of JPEG image compression.

It is expressed as percents - valid range: 1 to 100"#;

pub const LOSSLESS_HELP: &str = "Use only lossless compression for images.";
pub const LOSSLESS_LONG_HELP: &str = cstr!(
    r#"Use only lossless compression for images.

<s><i><y>Warning!</></></> This will dramatically increase the size of the output file!
"#
);

pub const LOG_HELP: &str = "Controls which messages are logged into console.";

pub const MARGIN_HELP: &str = "Margin for image pages.";

pub const FORCE_IMAGE_PAGE_FALLBACK_SIZE_HELP: &str =
    "Force the fallback size for image pages, overriding other PDFs.";

pub const ALPHABETIC_FILE_SORTING_HELP: &str = "Sort paths alphabetically, ignoring input order.";
pub const ALPHABETIC_FILE_SORTING_LONG_HELP: &str = r#"Sort paths alphabetically, ignoring input order.

By default files will be merged in the same order as they were specified.
When this flag is enabled, after collecting all files they will be sorted by their paths.
Useful when drag&dropping items onto the executable (as order in drag&drop may not be obvious)."#;

pub const LIBREOFFICE_PATH_HELP: &str = "Paths to LibreOffice executables for document conversion.";
pub const LIBREOFFICE_PATH_LONG_HELP: &str = r#"Paths to LibreOffice executables for document conversion.

If the paths are not available, LibreOffice conversion will be disabled."#;

pub const OUTPUT_DIRECTORY_HELP: &str = "Directory for output files.";
pub const OUTPUT_DIRECTORY_LONG_HELP: &str = r#"Directory for output files.

The output file will have a unique name based on the current date time.
The exact name of the file also depends on chosen language."#;

pub const OUTPUT_FILE_HELP: &str = "Path for the output file.";
pub const OUTPUT_FILE_LONG_HELP: &str = r#"Path for the output file.

The output will try to overwrite any existing file."#;

pub const ABOUT: &str =
    "Command-line tool to concatenate images, documents and PDFs into a single PDF file.";

pub const AFTER_HELP: &str = cstr!("Specify  <s><i>--help</></> to get detailed help and information about specifying sizes and reusing configurations.");
pub const AFTER_LONG_HELP: &str = cstr!(
    r#"<bright-cyan><s><i>Reusing configuration</></></>

It is possible to save a commonly used configuration to simplify commandline usage.

By default the application will attempt loading a configuration stored in "config.toml" near the executable.
If this file is missing, default values for all parameters will be used.

However, it is possible to specify a different configuration file(with <s><i>--config</></>). This file must exist.
The configuration file may contain any choice of available parameters. Missing parameters will use default values.
The loaded configuration (or default values) are then overwritten by all parameters specified in the commandline. 

For example, if the configuration file contains the following:
─────────────────────┐
log = "debug"        │
recursion_limit = 10 │
─────────────────────┘
and the command-line has these options: --recursion_limit 5 --lossless
the final configuration will still use "debug" logs, but with recursion limit of 5 and lossless quality.

If you specify the option <s><i>--save-config</></> with some filepath the used configuration will be saved for future use.
Some options are ignored in configuration files:
 - <s><i>--save-config</></>
 - <s><i>--config</></>
 - <s><i>--whatif</></>
 - <s><i>files</></>

 <s><i>TIP</></> If you want to use the program in a drag&drop manner you can create a shortcut that will set --config to a prepared file for drag&drop operations.
 For example, it may set the output directory to the desktop.

<bright-cyan><s><i>Sizes</></></>

Sizes can be expressed in two ways:

1) With one or two dimensions (width by height) separated by at least one 'x', space or '-'
    - If only width is specified, height is given the same value.
    - Supported units (units can be mixed:
        - Meters <s><i>m</></>
        - Millimers <s><i>mm</></>
        - Centimeters <s><i>cm</></>
        - Inches <s><i>in</></>
        - Points <s><i>pt</></>
    - Examples:
        - 5 mm x 7 mm
        - 5mm x 7mm
        - 5mmx7mm
        - 5mm-7mm
        - 5mm 7mm
        - 1in
        - 5mm x 2cm
2) As an ISO sheet, which will be then translated to standard dimensions.
    - Supported formats A, B, C
    - Supported ranks 0-13
    - Examples: A4, B5, C7
        - A4 
        - B5 
        - C11"#
);
