type Tag = u32;

// Universal class tag assignments at Rec. ITU-T X.680, clause 8, table 1
// pub const TAG_EOC: Tag = 0x0;
pub const TAG_BOOLEAN: Tag = 0x1;
pub const TAG_INTEGER: Tag = 0x2;
pub const TAG_BIT_STRING: Tag = 0x3;
pub const TAG_OCTET_STRING: Tag = 0x4;
pub const TAG_NULL: Tag = 0x5;
pub const TAG_OID: Tag = 0x6;
// pub const TAG_OBJECT_DESC: Tag = 0x7;
// pub const TAG_EXTERNAL: Tag = 0x8;
// pub const TAG_REAL: Tag = 0x9;
pub const TAG_ENUMERATED: Tag = 0xa;
// pub const TAG_EMBEDDED_PDV: Tag = 0xb;
pub const TAG_UTF8_STRING: Tag = 0xc;
// pub const TAG_RELATIVE_OID: Tag = 0xd;
// pub const TAG_TIME: Tag = 0xe;
// 0xf is reserved
pub const TAG_SEQUENCE: Tag = 0x10;
pub const TAG_SET: Tag = 0x11;
// pub const TAG_NUMERIC_STRING: Tag = 0x12;
// pub const TAG_PRINTABLE_STRING: Tag = 0x13;
// pub const TAG_T61_STRING: Tag = 0x14;
// pub const TAG_VIDEOTEX_STRING: Tag = 0x15;
// pub const TAG_IA5_STRING: Tag = 0x16;
// pub const TAG_UTC_TIME: Tag = 0x17;
// pub const TAG_GENERALIZED_TIME: Tag = 0x18;
// pub const TAG_GRAPHIC_STRING: Tag = 0x19;
// pub const TAG_VISIBLE_STRING: Tag = 0x1a;
// pub const TAG_GENERAL_STRING: Tag = 0x1b;
// pub const TAG_UNIVERSAL_STRING: Tag = 0x1c;
// pub const TAG_CHARACTER_STRING: Tag = 0x1d;
// pub const TAG_BMP_STRING: Tag = 0x1e;
// pub const TAG_DATE: Tag = 0x1f;
// pub const TAG_TIMEOFDAY: Tag = 0x20;
// pub const TAG_DATETIME: Tag = 0x21;
// pub const TAG_DURATION: Tag = 0x22;
// pub const TAG_OID_IRI: Tag = 0x23;
// pub const TAG_RELATIVE_OID_IRI: Tag = 0x23;