use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use clap::Parser;

mod fil0fil;
mod page0types;
mod rem0rec;
mod ut;

use crate::fil0fil::*;
use crate::page0types::*;
use crate::rem0rec::rec_get_next_offs;
use crate::ut::{fil_page_get_type, mach_read_from_2, mach_read_from_4, PAGE_SIZE};

/// Constants copied from Mysql InnoDB source code

/** The byte offsets on a file page for various variables. */

/** MySQL-4.0.14 space id the page belongs to (== 0) but in later
versions the 'new' checksum of the page */
const _FIL_PAGE_SPACE_OR_CHKSUM: u32 = 0;

/** page offset inside space */
const FIL_PAGE_OFFSET: u32 = 4;

/** if there is a 'natural' predecessor of the page, its offset.
Otherwise FIL_NULL. This field is not set on BLOB pages, which are stored as a
singly-linked list. See also FIL_PAGE_NEXT. */
const FIL_PAGE_PREV: u32 = 8;

/** On page 0 of the tablespace, this is the server version ID */
const FIL_PAGE_SRV_VERSION: u32 = 8;

/** if there is a 'natural' successor of the page, its offset. Otherwise
FIL_NULL. B-tree index pages(FIL_PAGE_TYPE contains FIL_PAGE_INDEX) on the
same PAGE_LEVEL are maintained as a doubly linked list via FIL_PAGE_PREV and
FIL_PAGE_NEXT in the collation order of the smallest user record on each
page. */
const FIL_PAGE_NEXT: u32 = 12;

/** On page 0 of the tablespace, this is the server version ID */
const _FIL_PAGE_SPACE_VERSION: u32 = 12;

/** lsn of the end of the newest modification log record to the page */
const _FIL_PAGE_LSN: u32 = 16;

/** file page type: FIL_PAGE_INDEX,..., 2 bytes. The contents of this field
can only be trusted in the following case: if the page is an uncompressed
B-tree index page, then it is guaranteed that the value is FIL_PAGE_INDEX.
The opposite does not hold.

In tablespaces created by MySQL/InnoDB 5.1.7 or later, the contents of this
field is valid for all uncompressed pages. */
const FIL_PAGE_TYPE: u32 = 24;

/** this is only defined for the first page of the system tablespace: the file
has been flushed to disk at least up to this LSN. For FIL_PAGE_COMPRESSED
pages, we store the compressed page control information in these 8 bytes. */
const _FIL_PAGE_FILE_FLUSH_LSN: u32 = 26;

/** If page type is FIL_PAGE_COMPRESSED then the 8 bytes starting at
FIL_PAGE_FILE_FLUSH_LSN are broken down as follows: */

/** Control information version format (u8) */
const _FIL_PAGE_VERSION: u32 = _FIL_PAGE_FILE_FLUSH_LSN;

/** Compression algorithm (u8) */
const _FIL_PAGE_ALGORITHM_V1: u32 = _FIL_PAGE_VERSION + 1;

/** Original page type (u16) */
const _FIL_PAGE_ORIGINAL_TYPE_V1: u32 = _FIL_PAGE_ALGORITHM_V1 + 1;

/** Original data size in bytes (u16)*/
const _FIL_PAGE_ORIGINAL_SIZE_V1: u32 = _FIL_PAGE_ORIGINAL_TYPE_V1 + 2;

/** Size after compression (u16) */
const _FIL_PAGE_COMPRESS_SIZE_V1: u32 = _FIL_PAGE_ORIGINAL_SIZE_V1 + 2;

/** This overloads FIL_PAGE_FILE_FLUSH_LSN for RTREE Split Sequence Number */
const _FIL_RTREE_SPLIT_SEQ_NUM: u32 = _FIL_PAGE_FILE_FLUSH_LSN;

/** starting from 4.1.x this contains the space id of the page */
const FIL_PAGE_ARCH_LOG_NO_OR_SPACE_ID: u32 = 34;

/** alias for space id */
const _FIL_PAGE_SPACE_ID: u32 = FIL_PAGE_ARCH_LOG_NO_OR_SPACE_ID;

/** start of the data on the page */
const FIL_PAGE_DATA: u32 = PAGE_HEADER;

/** File page trailer */
/** the low 4 bytes of this are used to store the page checksum, the
last 4 bytes should be identical to the last 4 bytes of FIL_PAGE_LSN */
const _FIL_PAGE_END_LSN_OLD_CHKSUM: u32 = 8;

/** size of the page trailer */
const _FIL_PAGE_DATA_END: u32 = 8;

/** First in address is the page offset. */
const _FIL_ADDR_PAGE: u64 = 0;

/** Then comes 2-byte byte offset within page.*/
const _FIL_ADDR_BYTE: u64 = 4;

/** Address size is 6 bytes. */
const _FIL_ADDR_SIZE: u64 = 6;

/** Path separator e.g., 'dir;...;dirN' */
const _FIL_PATH_SEPARATOR: u8 = b';';

const FSP_HEADER_OFFSET: u32 = FIL_PAGE_DATA;
const FSP_SPACE_ID: u32 = 0;

fn page_header_get_field(page_hdr: &[u8], field: u32) -> u16 {
    mach_read_from_2(&page_hdr[field as usize..])
}

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

fn page_is_comp(page_hdr: &[u8]) -> bool {
    (page_header_get_field(page_hdr, PAGE_N_HEAP) & 0x8000) != 0
}

fn print_page_header(buf: &mut [u8]) {
    let page_header = &mut buf[FIL_PAGE_DATA as usize..];

    println!("\nIndex Page Header Fields:");

    let page_directory_slots = page_header_get_field(page_header, PAGE_N_DIR_SLOTS);
    println!(
        "Number of slots in page directory: {}",
        page_directory_slots as u32
    );
    println!(
        "Pointer to record heap top: {}",
        page_header_get_field(page_header, PAGE_HEAP_TOP) as u32
    );
    println!(
        "Number of records in the heap: {}",
        page_header_get_field(page_header, PAGE_N_HEAP) as u32 & 0x7fff
    );
    println!(
        "Pointer to start of page free record list: {}",
        page_header_get_field(page_header, PAGE_FREE) as u32
    );
    println!(
        "Number of bytes in deleted records: {}",
        page_header_get_field(page_header, PAGE_GARBAGE) as u32
    );
    println!(
        "Pointer to the last inserted record: {}",
        page_header_get_field(page_header, PAGE_LAST_INSERT) as u32
    );
    println!(
        "Last insert direction: {}",
        page_header_get_field(page_header, PAGE_DIRECTION) as u32
    );
    println!(
        "Number of consecutive inserts to the same direction: {}",
        page_header_get_field(page_header, PAGE_N_DIRECTION) as u32
    );
    println!(
        "Number of user records on the page: {}",
        page_header_get_field(page_header, PAGE_N_RECS) as u32
    );
    println!(
        "Highest id of a trx which may have modified a record on the page: {}",
        page_header_get_field(page_header, PAGE_MAX_TRX_ID) as u32
    );
    println!(
        "Level of the node in an index tree: {}",
        page_header_get_field(page_header, PAGE_LEVEL) as u32
    );
    println!(
        "Index id where the page belongs: {}",
        page_header_get_field(page_header, PAGE_INDEX_ID) as u32
    );

    println!("Page is compact: {}", page_is_comp(page_header));
}

fn main() {
    let args = Args::parse();

    // Read file 16kb size block
    let mut file = File::open(args.file).expect("Failed to open file");
    let mut buf = [0u8; PAGE_SIZE as usize];

    file.seek(SeekFrom::Start(PAGE_SIZE as u64 * args.page_number as u64))
        .expect("Failed to seek to start");

    let mut rec_nr = args.num_records;
    if rec_nr == 0 {
        rec_nr = page_header_get_field(&buf[FIL_PAGE_DATA as usize..], PAGE_N_RECS) as u32;
    }

    println!("Reading Filespace Header:");
    file.read_exact(&mut buf)
        .expect("Unable to read file block");

    let fsp_id = mach_read_from_4(
        &buf[FIL_PAGE_ARCH_LOG_NO_OR_SPACE_ID as usize
            ..(FIL_PAGE_ARCH_LOG_NO_OR_SPACE_ID + 4) as usize],
    );

    println!("Filespace ID: {}", fsp_id);

    let page_no = mach_read_from_4(&buf[FIL_PAGE_OFFSET as usize..(FIL_PAGE_OFFSET + 4) as usize]);
    println!("Page Number: {}", page_no);

    let page_type = fil_page_get_type(&buf[FIL_PAGE_TYPE as usize..(FIL_PAGE_TYPE + 4) as usize]);
    println!("Page Type: {}", fil_print_page_type(page_type));

    if page_type == FIL_PAGE_TYPE_FSP_HDR {
        let serv_version = mach_read_from_4(
            &buf[FIL_PAGE_SRV_VERSION as usize..(FIL_PAGE_SRV_VERSION + 4) as usize],
        );
        println!("Server Version: {}", serv_version);

        let space_header = &mut buf[FSP_HEADER_OFFSET as usize..];
        let space_id = mach_read_from_4(&space_header[FSP_SPACE_ID as usize..]);

        println!("Space ID: {}", space_id);

        let space_size = mach_read_from_4(&space_header[8..]);
        println!("Space Size (in pages): {}", space_size);
    }

    if page_no == 0 {
        println!("Previous Page Number: FIL_NULL (This is the first page)");
    } else {
        let page_prev =
            mach_read_from_4(&buf[FIL_PAGE_PREV as usize..(FIL_PAGE_PREV + 4) as usize]);
        println!("Previous Page Number: {}", page_prev);
    }

    let page_next = mach_read_from_4(&buf[FIL_PAGE_NEXT as usize..(FIL_PAGE_NEXT + 4) as usize]);
    println!("Next Page Number: {}", page_next);

    print_page_header(&mut buf);

    let rec_infi = &buf[PAGE_NEW_INFIMUM as usize..(PAGE_NEW_INFIMUM + 8) as usize];
    println!("\nInfimum Record Data: {}", String::from_utf8_lossy(rec_infi));

    let rec_supremum = &buf[(PAGE_NEW_SUPREMUM) as usize..(PAGE_NEW_SUPREMUM_END) as usize];
    println!("\nSupremum Record Data: {}", String::from_utf8_lossy(rec_supremum));

    let mut off = PAGE_NEW_INFIMUM;
    let comp = page_is_comp(&buf) as u32;
    while rec_nr > 0 {
        let rec_start_off = rec_get_next_offs(&buf, off, comp);
        if rec_start_off > PAGE_SIZE as u32 {
            eprintln!("Record start offset exceeds page size...stopping.");
            break;
        }

        off += rec_start_off;
        let rec_end_off = rec_get_next_offs(&buf, off, comp);
        if rec_end_off > PAGE_SIZE as u32 {
            eprintln!("Record end offset exceeds page size...stopping.");
            break;
        }
        let rec = &buf[off as usize..(off + rec_end_off) as usize];
        println!("Data: {}", String::from_utf8_lossy(rec));
        rec_nr -= 1;
    }

    // buf.fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mach_read_from_2() {
        let buf = [0x12, 0x34];
        let result = mach_read_from_2(&buf);
        assert_eq!(result, 0x1234);
    }

    #[test]
    fn test_mach_read_from_4() {
        let buf = [0x12, 0x34, 0x56, 0x78];
        let result = mach_read_from_4(&buf);
        assert_eq!(result, 0x12345678);
    }

    #[test]
    fn test_fil_page_get_type() {
        let buf = [0x45, 0x67];
        let result = fil_page_get_type(&buf);
        assert_eq!(result, 0x4567);
    }

    #[test]
    fn test_page_header_get_field() {
        let buf = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let result = page_header_get_field(&buf, 2);
        assert_eq!(result, 0x0203);
    }
}
