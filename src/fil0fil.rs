/// Constants copied from Mysql InnoDB source code

/** The byte offsets on a file page for various variables. */
/** File page types (values of FIL_PAGE_TYPE) */
/** B-tree node */
pub const FIL_PAGE_INDEX: u16 = 17855;

/** R-tree node */
pub const FIL_PAGE_RTREE: u16 = 17854;

/** Tablespace SDI Index page */
pub const FIL_PAGE_SDI: u16 = 17853;

/** This page type is unused. */
pub const FIL_PAGE_TYPE_UNUSED: u16 = 1;

/** Undo log page */
pub const FIL_PAGE_UNDO_LOG: u16 = 2;

/** Index node */
pub const FIL_PAGE_INODE: u16 = 3;

/** Insert buffer free list */
pub const FIL_PAGE_IBUF_FREE_LIST: u16 = 4;

/* File page types introduced in MySQL/InnoDB 5.1.7 */
/** Freshly allocated page */
pub const FIL_PAGE_TYPE_ALLOCATED: u16 = 0;

/** Insert buffer bitmap */
pub const FIL_PAGE_IBUF_BITMAP: u16 = 5;

/** System page */
pub const FIL_PAGE_TYPE_SYS: u16 = 6;

/** Transaction system data */
pub const FIL_PAGE_TYPE_TRX_SYS: u16 = 7;

/** File space header */
pub const FIL_PAGE_TYPE_FSP_HDR: u16 = 8;

/** Extent descriptor page */
pub const FIL_PAGE_TYPE_XDES: u16 = 9;

/** Uncompressed BLOB page */
pub const FIL_PAGE_TYPE_BLOB: u16 = 10;

/** First compressed BLOB page */
pub const FIL_PAGE_TYPE_ZBLOB: u16 = 11;

/** Subsequent compressed BLOB page */
pub const FIL_PAGE_TYPE_ZBLOB2: u16 = 12;

/** In old tablespaces, garbage in FIL_PAGE_TYPE is replaced with
this value when flushing pages. */
pub const FIL_PAGE_TYPE_UNKNOWN: u16 = 13;

/** Compressed page */
pub const FIL_PAGE_COMPRESSED: u16 = 14;

/** Encrypted page */
pub const FIL_PAGE_ENCRYPTED: u16 = 15;

/** Compressed and Encrypted page */
pub const FIL_PAGE_COMPRESSED_AND_ENCRYPTED: u16 = 16;

/** Encrypted R-tree page */
pub const FIL_PAGE_ENCRYPTED_RTREE: u16 = 17;

/** Uncompressed SDI BLOB page */
pub const FIL_PAGE_SDI_BLOB: u16 = 18;

/** Compressed SDI BLOB page */
pub const FIL_PAGE_SDI_ZBLOB: u16 = 19;

/** Legacy doublewrite buffer page. */
pub const FIL_PAGE_TYPE_LEGACY_DBLWR: u16 = 20;

/** Rollback Segment Array page */
pub const FIL_PAGE_TYPE_RSEG_ARRAY: u16 = 21;

/** Index pages of uncompressed LOB */
pub const FIL_PAGE_TYPE_LOB_INDEX: u16 = 22;

/** Data pages of uncompressed LOB */
pub const FIL_PAGE_TYPE_LOB_DATA: u16 = 23;

/** The first page of an uncompressed LOB */
pub const FIL_PAGE_TYPE_LOB_FIRST: u16 = 24;

/** The first page of a compressed LOB */
pub const FIL_PAGE_TYPE_ZLOB_FIRST: u16 = 25;

/** Data pages of compressed LOB */
pub const FIL_PAGE_TYPE_ZLOB_DATA: u16 = 26;

/** Index pages of compressed LOB. This page contains an array of
z_index_entry_t objects.*/
pub const FIL_PAGE_TYPE_ZLOB_INDEX: u16 = 27;

/** Fragment pages of compressed LOB. */
pub const FIL_PAGE_TYPE_ZLOB_FRAG: u16 = 28;

/** Index pages of fragment pages (compressed LOB). */
pub const FIL_PAGE_TYPE_ZLOB_FRAG_ENTRY: u16 = 29;

/** Note the highest valid non-index page_type_t. */
pub const _FIL_PAGE_TYPE_LAST: u16 = FIL_PAGE_TYPE_ZLOB_FRAG_ENTRY;

pub const FIL_NULL: u32 = u32::MAX;

pub fn fil_print_page_type(page_type: u16) -> &'static str {
    match page_type {
        FIL_PAGE_INDEX => "FIL_PAGE_INDEX",
        FIL_PAGE_RTREE => "FIL_PAGE_RTREE",
        FIL_PAGE_SDI => "FIL_PAGE_SDI",
        FIL_PAGE_TYPE_UNUSED => "FIL_PAGE_TYPE_UNUSED",
        FIL_PAGE_UNDO_LOG => "FIL_PAGE_UNDO_LOG",
        FIL_PAGE_INODE => "FIL_PAGE_INODE",
        FIL_PAGE_IBUF_FREE_LIST => "FIL_PAGE_IBUF_FREE_LIST",
        FIL_PAGE_TYPE_ALLOCATED => "FIL_PAGE_TYPE_ALLOCATED",
        FIL_PAGE_IBUF_BITMAP => "FIL_PAGE_IBUF_BITMAP",
        FIL_PAGE_TYPE_SYS => "FIL_PAGE_TYPE_SYS",
        FIL_PAGE_TYPE_TRX_SYS => "FIL_PAGE_TYPE_TRX_SYS",
        FIL_PAGE_TYPE_FSP_HDR => "FIL_PAGE_TYPE_FSP_HDR",
        FIL_PAGE_TYPE_XDES => "FIL_PAGE_TYPE_XDES",
        FIL_PAGE_TYPE_BLOB => "FIL_PAGE_TYPE_BLOB",
        FIL_PAGE_TYPE_ZBLOB => "FIL_PAGE_TYPE_ZBLOB",
        FIL_PAGE_TYPE_ZBLOB2 => "FIL_PAGE_TYPE_ZBLOB2",
        FIL_PAGE_TYPE_UNKNOWN => "FIL_PAGE_TYPE_UNKNOWN",
        FIL_PAGE_COMPRESSED => "FIL_PAGE_COMPRESSED",
        FIL_PAGE_ENCRYPTED => "FIL_PAGE_ENCRYPTED",
        FIL_PAGE_COMPRESSED_AND_ENCRYPTED => "FIL_PAGE_COMPRESSED_AND_ENCRYPTED",
        FIL_PAGE_ENCRYPTED_RTREE => "FIL_PAGE_ENCRYPTED_RTREE",
        FIL_PAGE_SDI_BLOB => "FIL_PAGE_SDI_BLOB",
        FIL_PAGE_SDI_ZBLOB => "FIL_PAGE_SDI_ZBLOB",
        FIL_PAGE_TYPE_LEGACY_DBLWR => "FIL_PAGE_TYPE_LEGACY_DBLWR",
        FIL_PAGE_TYPE_RSEG_ARRAY => "FIL_PAGE_TYPE_RSEG_ARRAY",
        FIL_PAGE_TYPE_LOB_INDEX => "FIL_PAGE_TYPE_LOB_INDEX",
        FIL_PAGE_TYPE_LOB_DATA => "FIL_PAGE_TYPE_LOB_DATA",
        FIL_PAGE_TYPE_LOB_FIRST => "FIL_PAGE_TYPE_LOB_FIRST",
        FIL_PAGE_TYPE_ZLOB_FIRST => "FIL_PAGE_TYPE_ZLOB_FIRST",
        FIL_PAGE_TYPE_ZLOB_DATA => "FIL_PAGE_TYPE_ZLOB_DATA",
        FIL_PAGE_TYPE_ZLOB_INDEX => "FIL_PAGE_TYPE_ZLOB_INDEX",
        FIL_PAGE_TYPE_ZLOB_FRAG => "FIL_PAGE_TYPE_ZLOB_FRAG",
        FIL_PAGE_TYPE_ZLOB_FRAG_ENTRY => "FIL_PAGE_TYPE_ZLOB_FRAG_ENTRY",
        _ => "UNKNOWN_PAGE_TYPE",
    }
}
