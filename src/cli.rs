use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Http address
    #[arg(long, value_name = "URL")]
    pub address: String,

    /// Path to image folder (./images if not specified)
    #[arg(long, value_name = "FOLDER")]
    pub folder: Option<PathBuf>,

    /// Path to the Excel file
    #[arg(long, value_name = "FILE")]
    pub excel: PathBuf,

    /// Path to output file (current directory if not specified)
    #[arg(long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}
