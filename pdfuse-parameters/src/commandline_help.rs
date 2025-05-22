use color_print::cstr;
use indoc::indoc;

pub const FILES_HELP: &str = "Directories and files to be processed.";
pub const FILES_LONG_HELP: &str = r#"
            Directories and files to be processed.
            
            Both directory paths and file paths are acceptable.
            If a directory is provided PDFuse will look for applicable file recursively until depth of --recursion-limit is reached.
        "#;

pub const SAVE_CONFIG_HELP: &str =
    "Save input parameters as a TOML configuration file or send to stdout with `-`.";

pub const CONFIRM_EXIT_HELP: &str = "Require user input before closing the app.";

pub const WHAT_IF_HELP: &str = "Dry run: run the program without outputting files.";

pub const LANGUAGE_HELP: &str = "Specify a language file identifier.";

pub const CONFIG_HELP: &str = "Path to a custom configuration file.";

pub const RECURSION_LIMIT_HELP: &str = "Recursion depth limit for directories.";
pub const RECURSION_LIMIT_LONG_HELP: &str = "Recursion depth limit for directories.\nLimit of 0 mean only the files in the specified directory are scanned.\nLimit of 1 means the files in any immediate subfolders are also scanned, and so on.";

pub const IMAGE_PAGE_FALLBACK_SIZE_HELP: &str = "Fallback page size when adding images.";
pub const IMAGE_PAGE_FALLBACK_SIZE_LONG_HELP: &str = r#"
                Fallback page size when adding images.

                Images will usually use the same page size as the preceding document or PDF.
                If nothing precedes an image it will be placed on page size equal to the fallback size.

                Information about how to express a page size can be found in the SIZING section.
            "#;

pub const DPI_HELP: &str = "DPI used when adding images.";

pub const QUALITY_HELP: &str = "Quality of JPEG image compression.";
pub const QUALITY_LONG_HELP: &str = r#"
                Quality of JPEG image compression.
                It is expressed as percents - valid range: 1 to 100
            "#;

pub const LOSSLESS_HELP: &str = "Use only lossless compression for images.";
pub const LOSSLESS_LONG_HELP: &str = "
                Use only lossless compression for images.\
                \n
                \x1b[1m\x1b[5mWarning!\x1b[0m This will dramatically increase the size of the output file!
            ";

pub const LOG_HELP: &str = "Controls which messages are logged into console.";

pub const MARGIN_HELP: &str = "Margin for image pages.";

pub const FORCE_IMAGE_PAGE_FALLBACK_SIZE_HELP: &str =
    "Force the fallback size for image pages, overriding other PDFs.";

pub const ALPHABETIC_FILE_SORTING_HELP: &str = "Sort paths alphabetically, ignoring input order.";
pub const ALPHABETIC_FILE_SORTING_LONG_HELP: &str = r#"
                Sort paths alphabetically, ignoring input order.

                By default files will be merged in the same order as they were specified.
                When this flag is enabled, after collecting all files they will be sorted by their paths.
                Useful when drag&dropping items onto the executable (as order in drag&drop may not be obvious).
            "#;

pub const LIBREOFFICE_PATH_HELP: &str = "Paths to LibreOffice executables for document conversion.";
pub const LIBREOFFICE_PATH_LONG_HELP: &str = r#"
                Paths to LibreOffice executables for document conversion.
                
                If the paths are not available, LibreOffice conversion will be disabled.
            "#;

pub const OUTPUT_DIRECTORY_HELP: &str = "Directory for output files.";
pub const OUTPUT_DIRECTORY_LONG_HELP: &str = r#"
                Directory for output files.

                The output file will have a unique name based on the current date time.
                The exact name of the file also depends on chosen language.
            "#;

pub const OUTPUT_FILE_HELP: &str = "Path for the output file.";
pub const OUTPUT_FILE_LONG_HELP: &str = r#"
                Path for the output file.

                The output will try to overwrite any existing file.
            "#;

pub const ABOUT: &str =
    "Command-line tool to concatenate images, documents and PDFs into a single PDF file.";

pub const SIZING_SECTION: &str = r#"
            SIZING

            Sizes can be expressed in two ways:
            
             1) With one or two dimensions (width by height) separated by at least one 'x', space or '-'
                 • If only width is specified, height is given the same value.
                 • Supported units (units can be mixed:
                     - Meters m
                     - Millimers mm
                     - Centimeters cm
                     - Inches in
                     - Points pt
                 • Examples:
                     - 5 mm x 7 mm
                     - 5mm x 7mm
                     - 5mmx7mm
                     - 5mm-7mm
                     - 5mm 7mm
                     - 1in
                     - 5mm x 2cm
             2) As an ISO sheet, which will be then translated to standard dimensions.
                 • Supported formats A, B, C
                 • Supported ranks 0-13
                 • Examples: A4, B5, C7
                     - A4 
                     - B5 
                     - C7 
        "#;
