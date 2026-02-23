use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use core::ffi::{c_uint, c_ulong, c_void};
use flate2::Decompress;
use libz_sys::{
    Z_BUF_ERROR, Z_FINISH, Z_NO_FLUSH, Z_OK, Z_STREAM_END, inflate, inflateEnd, inflateInit_,
    z_stream, zlibVersion,
};
use std::ptr;

pub mod fil0fil;
pub mod page0types;
pub mod rem0rec;
pub mod ut;

use fil0fil::*;
use page0types::*;
use rem0rec::{REC_N_NEW_EXTRA_BYTES, rec_get_next_offs};
use ut::{
    _mach_read_from_1, PAGE_SIZE, fil_page_get_type, mach_read_from_2, mach_read_from_4,
    mach_read_from_8,
};

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

// -- Serialized dictionary information(SDI) --
/** Stored at rec_origin minus 2nd byte and length 2 bytes. */
const REC_OFF_NEXT: u32 = 2;

/** Stored at rec origin minus 3rd byte. Only 3bits of 3rd byte are used for
rec type. */
const REC_OFF_TYPE: u32 = 3;

/** Length of TYPE field in record of SDI Index. */
const REC_DATA_TYPE_LEN: u32 = 4;

/** Length of ID field in record of SDI Index. */
const REC_DATA_ID_LEN: u32 = 8;

/** SDI BLOB not expected before the following page number.
0 (tablespace header), 1 (tabespace bitmap), 2 (ibuf bitmap)
3 (SDI Index root page) */
const SDI_BLOB_ALLOWED: u64 = 4;

const REC_STATUS_INFIMUM: u32 = 2;
const REC_STATUS_SUPREMUM: u32 = 3;
const REC_OFF_DATA_TYPE: u32 = 0;
const REC_OFF_DATA_ID: u32 = REC_OFF_DATA_TYPE + REC_DATA_TYPE_LEN;
const REC_DATA_COMP_LEN: u32 = 4;

const DATA_TRX_ID_LEN: u32 = 6;
const DATA_ROLL_PTR_LEN: u32 = 7;
const REC_OFF_DATA_TRX_ID: u32 = REC_OFF_DATA_ID + REC_DATA_ID_LEN;
const REC_OFF_DATA_ROLL_PTR: u32 = REC_OFF_DATA_TRX_ID + DATA_TRX_ID_LEN;
const REC_DATA_UNCOMP_LEN: u32 = 4;

const REC_OFF_DATA_UNCOMP_LEN: u32 = REC_OFF_DATA_ROLL_PTR + DATA_ROLL_PTR_LEN;
const REC_OFF_DATA_COMP_LEN: u32 = REC_OFF_DATA_UNCOMP_LEN + REC_DATA_UNCOMP_LEN;
const REC_MIN_HEADER_SIZE: u32 = REC_N_NEW_EXTRA_BYTES;
const BTR_EXTERN_LEN: u32 = 12;
/** page number where stored */
const BTR_EXTERN_PAGE_NO: u32 = 4;
const REC_OFF_DATA_VARCHAR: u32 = REC_OFF_DATA_COMP_LEN + REC_DATA_COMP_LEN;

const FIL_ADDR_SIZE: u64 = 6;
const FLST_BASE_NODE_SIZE: u32 = 4 + 2 * FIL_ADDR_SIZE as u32;
const FSP_HEADER_SIZE: u32 = 32 + 5 * FLST_BASE_NODE_SIZE;
const FLST_NODE_SIZE: u32 = 2 * FIL_ADDR_SIZE as u32;
const XDES_BITMAP: u32 = FLST_NODE_SIZE + 12;
const XDES_ARR_OFFSET: u32 = FSP_HEADER_OFFSET + FSP_HEADER_SIZE;
/** How many bits are there per page */
const XDES_BITS_PER_PAGE: u32 = 2;

const FSP_EXTENT_SIZE: u32 = 64; // number of pages in an extent

/** Encryption magic bytes size */
const MAGIC_SIZE: u64 = 3;

/** Encryption key length */
const KEY_LEN: u64 = 32;

/** UUID of server instance, it's needed for composing master key name */
const SERVER_UUID_LEN: u64 = 36;

/** Encryption information total size: magic number + master_key_id +
key + iv + server_uuid + checksum */
const INFO_SIZE: u64 = MAGIC_SIZE
    + size_of::<u32>() as u64
    + (KEY_LEN * 2)
    + SERVER_UUID_LEN
    + size_of::<u32>() as u64;

/** Maximum size of Encryption information considering all
formats v1, v2 & v3. */
const INFO_MAX_SIZE: u64 = INFO_SIZE + size_of::<u32>() as u64;

/** Determine how many bytes (groups of 8 bits) are needed to
store the given number of bits. */
const fn ut_bits_in_bytes(bits: u32) -> u32 {
    (bits + 7) / 8 // Round up to the nearest byte
}

const fn xdes_arr_size(page_size: u32) -> u32 {
    page_size / FSP_EXTENT_SIZE
}

/** File extent data structure size in bytes. */
const XDES_SIZE: u32 = XDES_BITMAP + ut_bits_in_bytes(FSP_EXTENT_SIZE * XDES_BITS_PER_PAGE);

const fn fsp_header_get_sdi_offset(page_size: u32) -> u32 {
    XDES_ARR_OFFSET + XDES_SIZE * xdes_arr_size(page_size) + INFO_MAX_SIZE as u32
}

const FSP_FLAGS_MASK_ZIP_SSIZE: u32 = 30;
const FSP_FLAGS_POS_ZIP_SSIZE: u32 = 1;

const FSP_SPACE_FLAGS: u32 = 16;

const fn fsp_flags_get_zip_ssize(flags: u32) -> u32 {
    (flags & FSP_FLAGS_MASK_ZIP_SSIZE) >> FSP_FLAGS_POS_ZIP_SSIZE
}

/** SDI version. Written on Page 1 & 2 at FIL_PAGE_FILE_FLUSH_LSN offset. */
const SDI_VERSION: u32 = 1;

/** Offset within header of LOB length on this page. */
const LOB_HDR_PART_LEN: u32 = 0;

/** Size of an uncompressed LOB page header, in bytes */
const LOB_HDR_SIZE: u32 = 8;

/** Offset within header of next BLOB part page no.
FIL_NULL if none */
const LOB_HDR_NEXT_PAGE_NO: u32 = 4;

enum ERR {
    FAILURE = 1,
    SUCCESS = 0,
    NORECORDS = 2,
}
// -- SDI --

fn page_header_get_field(page_hdr: &[u8], field: u32) -> u16 {
    mach_read_from_2(&page_hdr[field as usize..])
}

fn page_is_comp(page: &[u8]) -> bool {
    (page_header_get_field(page, PAGE_HEADER + PAGE_N_HEAP) & 0x8000) != 0
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

    println!("Page is compact: {}", page_is_comp(buf));
}

fn read_page(page_number: u32, ibd_file: &mut File, buf: &mut [u8]) {
    ibd_file
        .seek(SeekFrom::Start(PAGE_SIZE as u64 * page_number as u64))
        .expect("Failed to seek to start");

    ibd_file.read_exact(buf).expect("Unable to read file block");
}

fn read_page_and_return_level(buf: &mut [u8], ibd_file: &mut File, page_number: u32) -> u64 {
    read_page(page_number, ibd_file, buf);
    let page_type = fil_page_get_type(&buf[FIL_PAGE_TYPE as usize..(FIL_PAGE_TYPE + 4) as usize]);
    if page_type != FIL_PAGE_SDI {
        return u64::MAX;
    }
    mach_read_from_2(&buf[(FIL_PAGE_DATA + PAGE_LEVEL) as usize..]) as u64
}

/* Get leaf with the lowest record to start scanning from.*/
fn reach_to_leftmost_leaf_level(buf: &mut [u8], root_page_number: u32, ibd_file: &mut File) -> ERR {
    let mut page_level = read_page_and_return_level(buf, ibd_file, root_page_number);
    if page_level >= u64::MAX {
        eprintln!("The provided root page is not an SDI page.");
        return ERR::FAILURE;
    }

    let num_recs = page_header_get_field(&buf, FIL_PAGE_DATA + PAGE_N_RECS) as u32;
    if num_recs == 0 {
        eprintln!("No records found on the SDI root page.");
        return ERR::NORECORDS;
    }

    if page_level == 0 {
        println!("Reached leaf level: {}", page_level);
        return ERR::SUCCESS;
    }

    /* Get to leftmost leaf page from the root page by following
     * child pointers(internal pages), from the infimum records
     */
    while page_level != 0 && page_level != u64::MAX {
        if get_rec_type(&buf, PAGE_NEW_INFIMUM) != REC_STATUS_INFIMUM as u8 {
            eprintln!("Infimum record not found");
            break;
        }
        let next_rec_off =
            mach_read_from_2(&buf[(PAGE_NEW_INFIMUM - REC_OFF_NEXT) as usize..]) as u32;

        let child_page_number = mach_read_from_4(
            &buf[(PAGE_NEW_INFIMUM + next_rec_off + REC_DATA_TYPE_LEN + REC_DATA_ID_LEN)
                as usize..],
        );

        if child_page_number < SDI_BLOB_ALLOWED as u32 {
            eprintln!(
                "Invalid child page number encountered: {}",
                child_page_number
            );
            return ERR::FAILURE;
        }

        let cur_page_level = page_level;

        page_level = read_page_and_return_level(buf, ibd_file, child_page_number);
        if cur_page_level >= u64::MAX {
            eprintln!("The child page is not an SDI page.");
            return ERR::FAILURE;
        }

        if page_level != cur_page_level - 1 {
            break;
        }
    }

    if page_level != 0 {
        eprintln!("Leftmost leaf level page not found or invalid.");
        return ERR::FAILURE;
    }

    ERR::SUCCESS
}

fn get_rec_type(page: &[u8], rec: u32) -> u8 {
    _mach_read_from_1(&page[(rec - REC_OFF_TYPE) as usize..]) & 0x7
}

fn get_first_user_rec(page: &[u8]) -> u32 {
    let next_rec_offset =
        mach_read_from_2(&page[(PAGE_NEW_INFIMUM - REC_OFF_NEXT) as usize..]) as u32;
    // rec_get_next_offs(page, PAGE_NEW_INFIMUM, page_is_comp(page) as u32);
    PAGE_NEW_INFIMUM + next_rec_offset
}

fn get_next_rec(page: &[u8], rec: u32, ibd_file: &mut File) -> u32 {
    let next_rec_offset: u32 = rec_get_next_offs(page, rec, page_is_comp(page) as u32);

    if next_rec_offset == 0 || next_rec_offset >= PAGE_SIZE as u32 {
        eprintln!(
            "Invalid offset encountered for next record: {} is more than the page size({})",
            next_rec_offset, PAGE_SIZE
        );
        return 0;
    }

    let mut next_rec = rec + next_rec_offset;

    if get_rec_type(&page, next_rec) == REC_STATUS_SUPREMUM as u8 {
        let next_page_num = mach_read_from_4(&page[FIL_PAGE_NEXT as usize..]);
        if next_page_num == FIL_NULL {
            return 0;
        }

        let buf = &mut [0u8; PAGE_SIZE as usize];
        read_page(next_page_num, ibd_file, buf);

        let page_type = fil_page_get_type(&buf[FIL_PAGE_TYPE as usize..]);
        if page_type != FIL_PAGE_SDI {
            eprintln!("Next page is not an SDI page.");
            return 0;
        }

        next_rec = get_first_user_rec(buf);
    }

    return next_rec;
}

fn copy_uncompressed_blob(
    first_blob_page_num: u32,
    _total_off_page_length: u64,
    buf: &mut [u8],
    ibd_file: &mut File,
) -> u64 {
    let page_buf = &mut [0u8; PAGE_SIZE as usize];
    let mut calc_length: u64 = 0;
    let mut part_len: u64;
    let mut next_page_num = first_blob_page_num;

    while next_page_num != FIL_NULL {
        read_page(next_page_num, ibd_file, page_buf);

        if fil_page_get_type(&page_buf[FIL_PAGE_TYPE as usize..]) != FIL_PAGE_SDI_BLOB {
            eprintln!("Unexpected uncompressed BLOB page type found");
            break;
        }

        part_len =
            mach_read_from_4(&page_buf[(FIL_PAGE_DATA + LOB_HDR_PART_LEN) as usize..]) as u64;
        buf[calc_length as usize..part_len as usize].copy_from_slice(
            &page_buf[(calc_length + (FIL_PAGE_DATA + LOB_HDR_SIZE) as u64) as usize
                ..(calc_length + (FIL_PAGE_DATA + LOB_HDR_SIZE) as u64 + part_len) as usize],
        );

        calc_length += part_len;

        next_page_num =
            mach_read_from_4(&page_buf[(FIL_PAGE_DATA + LOB_HDR_NEXT_PAGE_NO) as usize..]);

        if next_page_num as u64 <= SDI_BLOB_ALLOWED {
            eprintln!("Unexpected next blob number");
            break;
        }
    }
    calc_length
}

#[link(name = "mem0")]
unsafe extern "C" {
    /** Allocate memory for zlib. Defined in mem0.c*/
    fn page_zip_zalloc(opaque: *mut c_void, items: c_uint, size: c_uint) -> *mut c_void;
    fn page_zip_free(opaque: *mut c_void, address: *mut c_void);
    fn mem_heap_create(ptr: *mut c_void, size: c_uint) -> *mut c_void;
    fn mem_heap_free(ptr: *mut c_void);
}

// Allocate enough for decompression buffer
const HEAP_SIZE: u32 = 40000;

fn copy_compressed_blob(
    first_blob_page_num: u32,
    total_off_page_length: u64,
    buf: &mut [u8],
    ibd_file: &mut File,
) -> u64 {
    let mut arr = [0u8; HEAP_SIZE as usize];
    let heap: *mut c_void;
    unsafe {
        heap = mem_heap_create(arr.as_mut_ptr() as *mut c_void, HEAP_SIZE);
    }
    let mut page_num = first_blob_page_num;
    let page_buf = &mut [0u8; PAGE_SIZE as usize];
    let mut stream = z_stream {
        next_in: ptr::null_mut(),
        avail_in: 0,
        total_in: 0,
        next_out: buf.as_mut_ptr(),
        avail_out: total_off_page_length as u32,
        total_out: 0,
        msg: ptr::null_mut(),
        state: ptr::null_mut(),
        zalloc: page_zip_zalloc,
        zfree: page_zip_free,
        opaque: heap,
        data_type: 0,
        adler: 0,
        reserved: 0,
    };

    unsafe {
        inflateInit_(
            &mut stream,
            zlibVersion(),
            std::mem::size_of::<z_stream>() as i32,
        );
    };

    loop {
        read_page(page_num, ibd_file, page_buf);

        if fil_page_get_type(&page_buf[FIL_PAGE_TYPE as usize..]) != FIL_PAGE_SDI_ZBLOB {
            eprintln!(
                "Unexpected compressed BLOB page({}) found, got type({})",
                page_num,
                fil_page_get_type(&page_buf[FIL_PAGE_TYPE as usize..])
            );
            break;
        }

        let next_page_num = mach_read_from_4(&page_buf[FIL_PAGE_NEXT as usize..]);
        stream.next_in = page_buf[FIL_PAGE_DATA as usize..].as_mut_ptr();
        stream.avail_in = PAGE_SIZE as u32 - FIL_PAGE_DATA;

        let res: i32;
        unsafe {
            res = inflate(&mut stream, Z_NO_FLUSH);
        };
        match res {
            Z_OK => {
                if stream.avail_out <= 0 {
                    unsafe {
                        inflateEnd(&mut stream);
                        mem_heap_free(heap);
                    };
                    return stream.total_out as u64;
                }
                break;
            }
            Z_BUF_ERROR => {
                unsafe {
                    inflateEnd(&mut stream);
                    mem_heap_free(heap);
                };
                return stream.total_out as u64;
            }
            Z_STREAM_END => {
                if next_page_num == FIL_NULL {
                    unsafe {
                        inflateEnd(&mut stream);
                        mem_heap_free(heap);
                    };
                    return stream.total_out as u64;
                }
            }
            e => {
                eprintln!(
                    "Inflate() of compressed BLOB page returned {}({:?})",
                    e, stream.msg
                );
            }
        }

        if (next_page_num == FIL_NULL) || (next_page_num as u64 <= SDI_BLOB_ALLOWED) {
            if stream.avail_in <= 0 {
                eprintln!("Unexpected end of compressed BLOB page");
            } else {
                unsafe {
                    match inflate(&mut stream, Z_FINISH) {
                        Z_STREAM_END => break,
                        Z_BUF_ERROR => break,
                        e => {
                            eprintln!(
                                "Inflate() of compressed BLOB page returned {}({:?})",
                                e, stream.msg
                            );
                        }
                    }
                }
            }
        }

        page_num = next_page_num;
    }

    unsafe {
        inflateEnd(&mut stream);
        mem_heap_free(heap);
    };
    return stream.total_out as u64;
}

pub fn get_all_recs_in_sdi_leaf_pages(
    root_page_number: u32,
    ibd_file: &mut File,
) -> Option<Vec<u8>> {
    let mut sdi_obj: Vec<u8> = Vec::new();
    sdi_obj.push(b'[');
    let page = &mut [0u8; PAGE_SIZE as usize];
    match reach_to_leftmost_leaf_level(page, root_page_number, ibd_file) {
        ERR::SUCCESS => {
            let mut rec = get_first_user_rec(page);
            let mut sdi_id: u64;
            let mut _sdi_type: u64;

            while rec != 0 && get_rec_type(page, rec) != REC_STATUS_SUPREMUM as u8 {
                _sdi_type = mach_read_from_4(&page[(rec + REC_OFF_DATA_TYPE) as usize..]) as u64;
                sdi_id = mach_read_from_8(&page[(rec + REC_OFF_DATA_ID) as usize..]) as u64;
                let sdi_uncomp_len =
                    mach_read_from_4(&page[(rec + REC_OFF_DATA_UNCOMP_LEN) as usize..]) as c_ulong;
                let sdi_comp_len =
                    mach_read_from_4(&page[(rec + REC_OFF_DATA_COMP_LEN) as usize..]);

                let rec_data_len_partial =
                    _mach_read_from_1(&page[(rec - REC_MIN_HEADER_SIZE - 1) as usize..]) as u32;
                let mut rec_data_in_page_len: u32 = 0;
                let mut rec_data_length: u64;
                let mut is_rec_data_external: bool = false;

                /* If size is below 255, it's stored in a single byte. If it exceeds 255,
                 * 127 is stored in one byte, 128 or more is stored in 2 bytes
                 */
                if (rec_data_len_partial & 0x80) != 0 {
                    rec_data_in_page_len = (rec_data_len_partial & 0x3f) << 8;

                    /* Check if data is stored external from the record page */
                    if (rec_data_len_partial & 0x40) != 0 {
                        is_rec_data_external = true;
                        rec_data_length = mach_read_from_8(
                            &page[(rec
                                + REC_OFF_DATA_VARCHAR
                                + rec_data_in_page_len
                                + BTR_EXTERN_LEN) as usize..],
                        );
                        rec_data_length += rec_data_in_page_len as u64;
                    } else {
                        rec_data_length =
                            _mach_read_from_1(&page[(rec - REC_MIN_HEADER_SIZE - 2) as usize..])
                                as u64;
                        rec_data_length += rec_data_in_page_len as u64;
                    }
                } else {
                    rec_data_length = rec_data_len_partial as u64;
                }

                let buf = &mut vec![0u8; rec_data_length as usize];
                let rec_data_origin = &page[(rec + REC_OFF_DATA_VARCHAR) as usize..];

                if is_rec_data_external {
                    if rec_data_in_page_len != 0 {
                        buf[..rec_data_in_page_len as usize]
                            .copy_from_slice(&rec_data_origin[..rec_data_length as u32 as usize]);
                    }

                    let first_blob_page_num = mach_read_from_4(
                        &page[(rec
                            + rec_data_in_page_len
                            + REC_OFF_DATA_VARCHAR
                            + BTR_EXTERN_PAGE_NO) as usize..],
                    );
                    let fsp_flags = page_header_get_field(buf, PAGE_HEADER + FSP_SPACE_FLAGS);
                    let ssize = fsp_flags_get_zip_ssize(fsp_flags as u32);

                    if ssize == 0 {
                        // Uncompressed
                        _ = copy_uncompressed_blob(
                            first_blob_page_num,
                            rec_data_length - rec_data_in_page_len as u64,
                            &mut buf[rec_data_in_page_len as usize..],
                            ibd_file,
                        );
                    } else {
                        // Compressed
                        _ = copy_compressed_blob(
                            first_blob_page_num,
                            rec_data_length - rec_data_in_page_len as u64,
                            &mut buf[rec_data_in_page_len as usize..],
                            ibd_file,
                        );
                    }
                } else {
                    buf[..].copy_from_slice(&rec_data_origin[..rec_data_length as usize]);
                }

                if rec_data_length != sdi_comp_len as u64 {
                    eprintln!(
                        "Compressed data length mismatch for record with ID {}: expected {}, got {}",
                        sdi_id, sdi_comp_len, rec_data_length
                    );
                    if sdi_obj.is_empty() {
                        return None;
                    }
                    if let Some(last_byte) = sdi_obj.last() {
                        if *last_byte == b',' {
                            sdi_obj.pop();
                        }
                    }
                    sdi_obj.push(b']');
                    return Some(sdi_obj);
                }

                /* SDI data is in compressed format. Decompress and get data
                 * into sufficiently sized buffer
                 */
                let mut uncompressed_sdi_buf = vec![0u8; sdi_uncomp_len as usize];

                let mut decomp = Decompress::new(true);
                decomp
                    .decompress(
                        &buf,
                        &mut uncompressed_sdi_buf,
                        flate2::FlushDecompress::None,
                    )
                    .expect("Decompression failed");
                sdi_obj.extend_from_slice(&uncompressed_sdi_buf);

                rec = get_next_rec(page, rec, ibd_file);
                if rec != 0 && get_rec_type(page, rec) != REC_STATUS_SUPREMUM as u8 {
                    sdi_obj.push(b',');
                }
            }
            if let Some(last_byte) = sdi_obj.last() {
                if *last_byte == b',' {
                    sdi_obj.pop();
                }
            }
            sdi_obj.push(b']');
            return Some(sdi_obj);
        }
        ERR::NORECORDS => {
            println!("No records found in SDI leaf page.");
            None
        }
        ERR::FAILURE => None,
    }
}

pub fn print_ibd_file_data(file: &PathBuf, page_number: u32, num_records: u32) {
    // Read file 16kb size block
    let mut file = File::open(file).expect("Failed to open file");
    let mut buf = [0u8; PAGE_SIZE as usize];

    file.seek(SeekFrom::Start(PAGE_SIZE as u64 * page_number as u64))
        .expect("Failed to seek to start");

    let mut rec_nr = num_records;
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
    println!(
        "\nInfimum Record Data: {}",
        String::from_utf8_lossy(rec_infi)
    );

    let rec_supremum = &buf[(PAGE_NEW_SUPREMUM) as usize..(PAGE_NEW_SUPREMUM_END) as usize];
    println!(
        "\nSupremum Record Data: {}",
        String::from_utf8_lossy(rec_supremum)
    );

    /* Print data in SDI tree leaf pages */
    if page_type == FIL_PAGE_SDI {
        let buf = &mut [0u8; PAGE_SIZE as usize];
        read_page(0, &mut file, buf);
        let sdi_offset = fsp_header_get_sdi_offset(PAGE_SIZE as u32);
        let version = mach_read_from_4(&buf[sdi_offset as usize..]);

        if version != SDI_VERSION {
            eprintln!(
                "Unsupported SDI version: {}. Expected version: {}",
                version, SDI_VERSION
            );
            return;
        }

        let sdi_root_page_number =
            mach_read_from_4(&buf[(sdi_offset + 4) as usize..(sdi_offset + 8) as usize]);
        let sdi_json = match get_all_recs_in_sdi_leaf_pages(sdi_root_page_number, &mut file) {
            Some(json) => json,
            None => {
                eprintln!("Failed to read SDI leaf pages or no records found.");
                return;
            }
        };
        println!("SDI Data: {}", String::from_utf8_lossy(&sdi_json));
    }

    /* Print records in leaf data pages only */
    if page_type == FIL_PAGE_INDEX {
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
                eprintln!(
                    "Record end offset({}) exceeds page size...stopping.",
                    rec_end_off
                );
                break;
            }
            let rec = &buf[off as usize..(off + rec_end_off) as usize];
            println!("Data: {}", String::from_utf8_lossy(rec));
            rec_nr -= 1;
        }
    }
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
