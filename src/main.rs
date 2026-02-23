use std::path::PathBuf;

use clap::Parser;

use mysql_ibdinspect::print_ibd_file_data;

#[derive(Debug, Parser)]
struct Args {
    /// Path to the InnoDB tablespace file (.ibd file)
    #[arg(short = 'f', long)]
    file: PathBuf,
    #[arg(short = 'n', long, default_value_t = 0)]
    page_number: u32,
    #[arg(short = 'r', long, default_value_t = 0)]
    num_records: u32,
}

fn main() {
    let args = Args::parse();

    print_ibd_file_data(&args.file, args.page_number, args.num_records);
}
