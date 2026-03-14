use crate::{Error, read_bytes::ReadBytesExt};

pub struct FileStruct;

impl FileStruct {
    pub fn read<R: ReadBytesExt>(mut reader: R) -> Result<Self, Error> {
        Ok(Self {})
    }
}
