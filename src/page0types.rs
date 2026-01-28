/// Constants copied from Mysql InnoDB source code

/*                      PAGE HEADER
                        ===========

Index page header starts at the first offset left free by the FIL-module */

/** index page header starts at this offset */
pub const PAGE_HEADER: u32 = 38; // same as FIL_PAGE_DATA

/** Length of the file system  header, in bytes */
const _FSEG_HEADER_SIZE: u32 = 10;

/*-----------------------------*/
/** number of slots in page directory */
pub const PAGE_N_DIR_SLOTS: u32 = 0;
/** pointer to record heap top */
pub const PAGE_HEAP_TOP: u32 = 2;
/** number of records in the heap, bit 15=flag: new-style compact page format */
pub const PAGE_N_HEAP: u32 = 4;
/** pointer to start of page free record list */
pub const PAGE_FREE: u32 = 6;
/** number of bytes in deleted records */
pub const PAGE_GARBAGE: u32 = 8;
/** pointer to the last inserted record, or NULL if this info has been reset by
 a delete, for example */
pub const PAGE_LAST_INSERT: u32 = 10;
/** last insert direction: PAGE_LEFT, ... */
pub const PAGE_DIRECTION: u32 = 12;
/** number of consecutive inserts to the same direction */
pub const PAGE_N_DIRECTION: u32 = 14;
/** number of user records on the page */
pub const PAGE_N_RECS: u32 = 16;
/** highest id of a trx which may have modified a record on the page; trx_id_t;
defined only in secondary indexes and in the insert buffer tree */
pub const PAGE_MAX_TRX_ID: u32 = 18;
/** end of private data structure of the page header which are set in a page
create */
pub const _PAGE_HEADER_PRIV_END: u32 = 26;
/*----*/
/** level of the node in an index tree; the leaf level is the level 0.
This field should not be written to after page creation. */
pub const PAGE_LEVEL: u32 = 26;
/** index id where the page belongs. This field should not be written to after
 page creation. */
pub const PAGE_INDEX_ID: u32 = 28;
/** file segment header for the leaf pages in a B-tree: defined only on the root
 page of a B-tree, but not in the root of an ibuf tree */
pub const _PAGE_BTR_SEG_LEAF: u32 = 36;
pub const _PAGE_BTR_IBUF_FREE_LIST: u32 = _PAGE_BTR_SEG_LEAF;
pub const _PAGE_BTR_IBUF_FREE_LIST_NODE: u32 = _PAGE_BTR_SEG_LEAF;
/* in the place of PAGE_BTR_SEG_LEAF and _TOP
there is a free list base node if the page is
the root page of an ibuf tree, and at the same
place is the free list node if the page is in
a free list */
pub const _PAGE_BTR_SEG_TOP: u32 = 36 + _FSEG_HEADER_SIZE;
/* file segment header for the non-leaf pages
in a B-tree: defined only on the root page of
a B-tree, but not in the root of an ibuf
tree */
/*----*/
/** start of data on the page */
pub const _PAGE_DATA: u32 = PAGE_HEADER + 36 + 2 * _FSEG_HEADER_SIZE;
