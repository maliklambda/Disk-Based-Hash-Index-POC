use std::{io, num::ParseIntError};

use crate::db::hash::IdxHash;

/// Error returned during execution of commands on the index.
#[derive(Debug)]
pub enum ExecuteError<'a> {
    GetErr(GetErr),
    InsertErr(InsertErr),
    DeleteErr(DeleteErr),
    UpdateErr(UpdateErr<'a>),
    AppendErr(AppendErr),
    InitErr(InitErr),
    ValueTooLong { max_len: usize, got: usize },
    KeyTooLong { max_len: usize, got: usize },
    ExitCmd,
}

impl<'a> From<GetErr> for ExecuteError<'a> {
    fn from(value: GetErr) -> Self {
        Self::GetErr(value)
    }
}

impl<'a> From<AppendErr> for ExecuteError<'a> {
    fn from(value: AppendErr) -> Self {
        Self::AppendErr(value)
    }
}

impl<'a> From<InsertErr> for ExecuteError<'a> {
    fn from(value: InsertErr) -> Self {
        Self::InsertErr(value)
    }
}

impl<'a> From<DeleteErr> for ExecuteError<'a> {
    fn from(value: DeleteErr) -> Self {
        Self::DeleteErr(value)
    }
}

impl<'a> From<UpdateErr<'a>> for ExecuteError<'a> {
    fn from(value: UpdateErr<'a>) -> Self {
        Self::UpdateErr(value)
    }
}

impl<'a> From<InitErr> for ExecuteError<'a> {
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
pub enum AppendErr {
    IoErr(io::Error),
}

impl From<io::Error> for AppendErr {
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
    InvalidLen,
    ByteConvertErr,
    IoErr(io::Error),
}

impl From<io::Error> for GetErr {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}

#[derive(Debug)]
pub enum DeleteErr {
    IoErr(io::Error),
    GetErr(GetErr),
}

impl From<io::Error> for DeleteErr {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}

impl From<GetErr> for DeleteErr {
    fn from(value: GetErr) -> Self {
        Self::GetErr(value)
    }
}

#[derive(Debug)]
pub enum UpdateErr<'a> {
    IoErr(io::Error),
    NonExistantValue(&'a str),
}

impl<'a> From<io::Error> for UpdateErr<'a> {
    fn from(value: io::Error) -> Self {
        Self::IoErr(value)
    }
}
