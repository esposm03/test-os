use crate::components::dynamic::DynamicTag;

use alloc::string::String;

use displaydoc::Display;

/// An error that occurred while trying to read relocations
#[derive(Display, Debug)]
pub enum ReadRelaError {
    /// Failed to get dynamic entry: {0}
    DynamicEntryNotFound(GetDynamicEntryError),
    /// Dynamic section not found
    DynamicSectionNotFound,
    /// Object file does not contain a `SHT_RELA` section
    RelaSegmentNotFound,
    /// Object file does not contain a `SHT_REL` section
    RelSegmentNotFound,
    /// Parsing error: {0}
    ParsingError(String),
}

#[derive(Display, Debug)]
pub enum GetDynamicEntryError {
    /// Dynamic entry {0:?} not found
    NotFound(DynamicTag),
}

/// An error that occurred while trying to read strings from the file
#[derive(Display, Debug)]
pub enum GetStringError {
    /// StrTab dynamic entry not found
    StrTabNotFound,
    /// StrTab segment not found
    StrTabSegmentNotFound,
    /// String not found
    StringNotFound,
}

/// An error that occurred while trying to read symbols
#[derive(Display, Debug)]
pub enum ReadSymsError {
    /// Dynamic entry not found: {0}
    DynamicEntryNotFound(GetDynamicEntryError),
    /// SymTab section not found
    SymTabSectionNotFound,
    /// SymTab segment not found
    SymTabSegmentNotFound,
    /// Parsing error: {0}
    ParsingError(String),
}
