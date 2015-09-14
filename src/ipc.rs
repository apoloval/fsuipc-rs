
//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// The header of a message sent to FSUIPC module via IPC
#[allow(raw_pointer_derive)]
#[derive(Debug, PartialEq)]
pub enum MsgHeader {
    /// Read state data message header
    /// Read `len` bytes from given offset and prepar to store `data` in `target`.
    ReadStateData {
        offset: u16,
        len: usize,
        target: *mut u8,
    },
    /// Write state data message header
    /// Write `len` bytes from given `source` to given offset.
    WriteStateData {
        offset: u16,
        len: usize,
    },
    TerminationMark
}

pub trait MsgHeaderRead : Read {
    /// Read a IPC message header from the given `Read` object.
    /// It returns the read message header and the number of bytes processed.
    fn read_header(&mut self) -> io::Result<(MsgHeader, usize)> {
        match try!(self.read_u32::<LittleEndian>()) {
            FS6IPC_READSTATEDATA_ID => {
                let offset = try!(self.read_u32::<LittleEndian>()) as u16;
                let len = try!(self.read_u32::<LittleEndian>()) as usize;
                let target = try!(self.read_u32::<LittleEndian>()) as *mut u8;
                Ok((MsgHeader::ReadStateData {
                    offset: offset,
                    len: len,
                    target: target,
                }, 16))
            },
            FS6IPC_WRITESTATEDATA_ID => {
                let offset = try!(self.read_u32::<LittleEndian>()) as u16;
                let len = try!(self.read_u32::<LittleEndian>()) as usize;
                Ok((MsgHeader::WriteStateData {
                    offset: offset,
                    len: len,
                }, 12))
            },
            FS6IPC_TERMINATIONMARK_ID => return Ok((MsgHeader::TerminationMark, 4)),
            unexpected => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unexpected double word 0x{} while reading IPC message header",
                    unexpected))),
        }
    }
}

impl<R: Read + ?Sized> MsgHeaderRead for R {}

pub trait MsgHeaderWrite : Write {
    /// Write a IPC message header into the given `Write` object.
    fn write_header(&mut self, msg: &MsgHeader) -> io::Result<usize> {
        match msg {
            &MsgHeader::ReadStateData { offset, len, target } => {
                try!(self.write_u32::<LittleEndian>(FS6IPC_READSTATEDATA_ID));
                try!(self.write_u32::<LittleEndian>(offset as u32));
                try!(self.write_u32::<LittleEndian>(len as u32));
                try!(self.write_u32::<LittleEndian>(target as u32));
                Ok(16)
            },
            &MsgHeader::WriteStateData { offset, len } => {
                try!(self.write_u32::<LittleEndian>(FS6IPC_WRITESTATEDATA_ID));
                try!(self.write_u32::<LittleEndian>(offset as u32));
                try!(self.write_u32::<LittleEndian>(len as u32));
                Ok(12)
            },
            &MsgHeader::TerminationMark => {
                try!(self.write_u32::<LittleEndian>(FS6IPC_TERMINATIONMARK_ID));
                Ok(4)
            },
        }
    }
}

impl<W: Write + ?Sized> MsgHeaderWrite for W {}

const FS6IPC_TERMINATIONMARK_ID: u32 = 0;
const FS6IPC_READSTATEDATA_ID: u32 = 1;
const FS6IPC_WRITESTATEDATA_ID: u32 = 2;

#[cfg(test)]
mod test {

    use std::io::{Cursor, ErrorKind};

    use byteorder::{LittleEndian, ReadBytesExt};

    use super::*;

    #[test]
    fn should_read_rsd() {
        let mut buff: &[u8] = &[
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x00, 0x20, 0x00, 0x00,
        ];
        let expected = MsgHeader::ReadStateData {
            offset: 0x1000,
            len: 4,
            target: 0x2000 as *mut u8,
        };
        assert_eq!(buff.read_header().unwrap(), (expected, 16));
    }

    #[test]
    fn should_read_wsd() {
        let mut buff: &[u8] = &[
            0x02, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
        ];
        let expected = MsgHeader::WriteStateData {
            offset: 0x1000,
            len: 4,
        };
        assert_eq!(buff.read_header().unwrap(), (expected, 12));
    }

    #[test]
    fn should_read_tm() {
        let mut buff: &[u8] = &[0x00, 0x00, 0x00, 0x00];
        assert_eq!(buff.read_header().unwrap(), ( MsgHeader::TerminationMark, 4));
    }

    #[test]
    fn should_fail_to_read_from_invalid_stream() {
        let mut buff: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        let expected_error = ErrorKind::InvalidInput;
        let actual_error = buff.read_header().err().unwrap().kind();
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn should_write_rsd() {
        let mut buff = Cursor::new(Vec::new());
        let msg = MsgHeader::ReadStateData {
            offset: 0x1000,
            len: 4,
            target: 0x2000 as *mut u8,
        };
        assert_eq!(buff.write_header(&msg).unwrap(), 16);
        buff.set_position(0);
        assert_eq!(buff.get_ref().len(), 16);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 1);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 0x1000);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 4);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 0x2000);
    }

    #[test]
    fn should_write_wsd() {
        let mut buff = Cursor::new(Vec::new());
        let msg = MsgHeader::WriteStateData {
            offset: 0x1000,
            len: 4,
        };
        assert_eq!(buff.write_header(&msg).unwrap(), 12);
        buff.set_position(0);
        assert_eq!(buff.get_ref().len(), 12);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 2);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 0x1000);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 4);
    }

    #[test]
    fn should_write_tm() {
        let mut buff = Cursor::new(Vec::new());
        assert_eq!(buff.write_header(&MsgHeader::TerminationMark).unwrap(),4);
        buff.set_position(0);
        assert_eq!(buff.read_u32::<LittleEndian>().unwrap(), 0);
    }
}
