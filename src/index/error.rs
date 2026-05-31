use std::{io, num::ParseIntError};

use crate::index::hash::IdxHash;

/// Error returned during execution of commands on the index.
#[derive(Debug)]
pub enum ExecuteError {
    GetErr(GetErr),
    InsertErr(InsertErr),
    InitErr(InitErr),
    ExitCmd,
}

impl From<GetErr> for ExecuteError {
    fn from(value: GetErr) -> Self {
        Self::GetErr(value)
    }
}

impl From<InsertErr> for ExecuteError {
    fn from(value: InsertErr) -> Self {
        Self::InsertErr(value)
    }
}

impl From<InitErr> for ExecuteError {
    fn from(value: InitErr) -> Self {
        Self::InitErr(value)
    }
}

#[derive(Debug)]
pub enum InitErr {
    IoErr(io::Error),
}

impl From<io::Error> for InitErr {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}

#[derive(Debug)]
pub enum InsertErr {
    IoErr(io::Error),
    CWErr(CollisionWalkErr),
    ParseErr(ParseIntError),
}

impl From<io::Error> for InsertErr {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}

impl From<CollisionWalkErr> for InsertErr {
    fn from(value: CollisionWalkErr) -> Self {
        Self::CWErr(value)
    }
}

#[derive(Debug)]
pub enum CollisionWalkErr {
    ByteConvertErr,
    HashNotFound { hash: IdxHash, len: u32 },
}

#[derive(Debug)]
pub enum GetErr {
    HashNotFound(IdxHash),
    ByteConvertErr,
    IoErr(io::Error),
}

impl From<io::Error> for GetErr {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}
