use crate::ut::{_mach_read_from_1, PAGE_SIZE, mach_read_from_2, ut_align_offset};

const REC_NEXT: u32 = 2;
pub const REC_N_NEW_EXTRA_BYTES: u32 = 5;
pub const _REC_N_OLD_EXTRA_BYTES: u32 = 6;
const REC_NEW_STATUS: u32 = 3;
const REC_NEW_STATUS_MASK: u32 = 0x7u32;
const REC_NEW_STATUS_SHIFT: u32 = 0;
pub const REC_OFFS_COMPACT: u32 = 1u32 << 31;
const REC_INFO_BITS_MASK: u32 = 0xF0u32;
const REC_INFO_BITS_SHIFT: u32 = 4;
const REC_OLD_INFO_BITS: u32 = 6;
const REC_NEW_INFO_BITS: u32 = 5;

// External flag in offsets returned by rec_get_offsets()
pub const REC_OFFS_EXTERNAL: u32 = 1 << 30;

/// SQL NULL flag in offsets returned by rec_get_offsets()
pub const REC_OFFS_SQL_NULL: u32 = 1u32 << 31;

/// Use this bit to indicate record has version
const REC_INFO_VERSION_FLAG: u32 = 0x40u32;

/// Length of the rec_get_offsets() header
pub const REC_OFFS_HEADER_SIZE: u32 = 4;

/// Record status values
pub const REC_STATUS_ORDINARY: u32 = 0;
// const REC_STATUS_NODE_PTR: u32 = 1;
// const REC_STATUS_INFIMUM: u32 = 2;
// const REC_STATUS_SUPREMUM: u32 = 3;

/// An INSTANT DROP flag in offsets returned by rec_get_offsets()
const REC_OFFS_DROP: u32 = 1 << 28;

/// Mask for offsets returned by rec_get_offsets() */
pub const REC_OFFS_MASK: u32 = REC_OFFS_DROP - 1;

#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
pub enum ColumnTypes {
    DECIMAL = 1, // This is 1 > than MYSQL_TYPE_DECIMAL
    TINY = 2,
    SHORT = 3,
    LONG = 4,
    FLOAT = 5,
    DOUBLE = 6,
    TypeNull = 7,
    TIMESTAMP = 8,
    LONGLONG = 9,
    INT24 = 10,
    DATE = 11,
    TIME = 12,
    DATETIME = 13,
    YEAR = 14,
    NEWDATE = 15,
    VARCHAR = 16,
    BIT = 17,
    TIMESTAMP2 = 18,
    DATETIME2 = 19,
    TIME2 = 20,
    NEWDECIMAL = 21,
    ENUM = 22,
    SET = 23,
    TinyBlob = 24,
    MediumBlob = 25,
    LongBlob = 26,
    BLOB = 27,
    VarString = 28,
    STRING = 29,
    GEOMETRY = 30,
    JSON = 31,
    VECTOR = 32,
}

impl From<u32> for ColumnTypes {
    fn from(value: u32) -> Self {
        match value {
            1 => ColumnTypes::DECIMAL,
            2 => ColumnTypes::TINY,
            3 => ColumnTypes::SHORT,
            4 => ColumnTypes::LONG,
            5 => ColumnTypes::FLOAT,
            6 => ColumnTypes::DOUBLE,
            7 => ColumnTypes::TypeNull,
            8 => ColumnTypes::TIMESTAMP,
            9 => ColumnTypes::LONGLONG,
            10 => ColumnTypes::INT24,
            11 => ColumnTypes::DATE,
            12 => ColumnTypes::TIME,
            13 => ColumnTypes::DATETIME,
            14 => ColumnTypes::YEAR,
            15 => ColumnTypes::NEWDATE,
            16 => ColumnTypes::VARCHAR,
            17 => ColumnTypes::BIT,
            18 => ColumnTypes::TIMESTAMP2,
            19 => ColumnTypes::DATETIME2,
            20 => ColumnTypes::TIME2,
            21 => ColumnTypes::NEWDECIMAL,
            22 => ColumnTypes::ENUM,
            23 => ColumnTypes::SET,
            24 => ColumnTypes::TinyBlob,
            25 => ColumnTypes::MediumBlob,
            26 => ColumnTypes::LongBlob,
            27 => ColumnTypes::BLOB,
            28 => ColumnTypes::VarString,
            29 => ColumnTypes::STRING,
            30 => ColumnTypes::GEOMETRY,
            31 => ColumnTypes::JSON,
            32 => ColumnTypes::VECTOR,
            _ => panic!("Unknown column type: {}", value),
        }
    }
}

pub(crate) fn get_fixed_column_size(col_type: ColumnTypes) -> u32 {
    match col_type {
        ColumnTypes::TINY => 1,
        ColumnTypes::SHORT => 2,
        ColumnTypes::LONG => 4,
        ColumnTypes::FLOAT => 4,
        ColumnTypes::DOUBLE => 8,
        ColumnTypes::TypeNull => 0,
        ColumnTypes::TIMESTAMP => 4,
        ColumnTypes::LONGLONG => 8,
        ColumnTypes::INT24 => 3,
        // Recheck these
        ColumnTypes::DATE => 3,
        ColumnTypes::TIME => 3,
        ColumnTypes::DATETIME => 8,
        ColumnTypes::YEAR => 1,
        ColumnTypes::NEWDATE => 3,
        _ => u32::MAX, // Variable length
    }
}

fn rec_get_bit_field_1(page: &[u8], rec: u32, offs: u32, mask: u32, shift: u32) -> u32 {
    let byte_val: u8 = _mach_read_from_1(&page[(rec - offs) as usize..]);
    (byte_val as u32 & mask) >> shift
}

pub(crate) fn rec_get_status(page: &[u8], rec: u32) -> u32 {
    rec_get_bit_field_1(
        page,
        rec,
        REC_NEW_STATUS,
        REC_NEW_STATUS_MASK,
        REC_NEW_STATUS_SHIFT,
    )
}

fn rec_get_info_bits(page: &[u8], rec: u32, comp: bool) -> u32 {
    let offs: u32 = if comp {
        REC_NEW_INFO_BITS
    } else {
        REC_OLD_INFO_BITS
    };

    rec_get_bit_field_1(page, rec, offs, REC_INFO_BITS_MASK, REC_INFO_BITS_SHIFT)
}

/// Tells if a new-style record is versioned.
fn rec_new_is_versioned(page: &[u8], rec: u32) -> bool {
    rec_get_info_bits(page, rec, true) & REC_INFO_VERSION_FLAG != 0
}

pub(crate) fn rec_get_next_offs(page: &[u8], rec: u32, comp: u32) -> u32 {
    rec_new_is_versioned(page, rec);
    if rec < REC_NEXT {
        return 0;
    }
    let field_val: u32 = mach_read_from_2(&page[(rec - REC_NEXT) as usize..]) as u32;
    if comp > 0 {
        if field_val == 0 {
            return 0;
        }
        return ut_align_offset(rec + field_val, PAGE_SIZE as u32);
    } else {
        return field_val;
    }
}
