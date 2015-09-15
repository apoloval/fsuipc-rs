
//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::CString;
use std::io;
use std::ptr;

use user32::{FindWindowExA, SendMessageA};
use winapi::WM_USER;
use winapi::windef::HWND;

use super::{Handle, Session};
use super::ipc::*;
use super::raw::MutRawBytes;

/// A handle to FSUIPc that uses local IPC communication to the FSUIPC module
/// This kind of handle must be used from code running in the same process as FSUIPC does.
pub struct LocalHandle {
    handle: HWND,
}

impl LocalHandle {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let win_name = CString::new("UIPCMAIN").unwrap();
            let handle = FindWindowExA(
                ptr::null_mut(), ptr::null_mut(), win_name.as_ptr(), ptr::null_mut());
            if handle != ptr::null_mut() {
                Ok(LocalHandle { handle: handle })
            } else {
                Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to local FSUIPC: cannot create window handle"))
            }
        }
    }
}

impl<'a> Handle<'a> for LocalHandle {
    type Sess = LocalSession;

    fn session(&'a mut self) -> LocalSession {
        LocalSession::new(self.handle)
    }
}

pub struct LocalSession {
    handle: HWND,
    buffer: io::Cursor<Vec<u8>>,
}

impl LocalSession {
    fn new(handle: HWND) -> Self {
        let mut session = LocalSession {
            handle: handle,
            buffer: io::Cursor::new(Vec::with_capacity(4096))
        };
        // First 4-bytes seems to be for a stack frame pointer that is not actually used
        session.buffer.set_position(4);
        session
    }
}

impl Session for LocalSession {
    fn read_bytes(&mut self, offset: u16, dest: *mut u8, len: usize) -> io::Result<usize> {
        self.buffer.write_rsd(offset, dest, len)
    }

    fn write_bytes(&mut self, offset: u16, src: *const u8, len: usize) -> io::Result<usize> {
        self.buffer.write_wsd(offset, src, len)
    }

    fn process(mut self) -> io::Result<usize> {
        unsafe {
            try!(self.buffer.write_header(&MsgHeader::TerminationMark));
            let nbytes = self.buffer.position() as usize;
            let buff = self.buffer.get_ref().as_ptr() as i32;
            let send_result = SendMessageA(self.handle, WM_IPCTHREADACCESS, nbytes as u32, buff);
            if send_result != FS6IPC_MESSAGE_SUCCESS {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                    "FSUIPC rejected the requests with error {}; possible buffer corruption in bytes: {:?}",
                    send_result, self.buffer.get_ref())));
            }
            // First 4-bytes seems to be for a stack frame pointer that is not actually used
            self.buffer.set_position(4);
            loop {
                let header = try!(self.buffer.read_header());
                match &header {
                    &MsgHeader::ReadStateData { offset: _, len, target } => {
                        let mut output = MutRawBytes::new(target, len);
                        try!(self.buffer.read_body(&header, &mut output));
                    },
                    &MsgHeader::WriteStateData { offset: _, len: _ } => {
                        let mut output = io::sink();
                        try!(self.buffer.read_body(&header, &mut output));
                    },
                    &MsgHeader::TerminationMark => return Ok(nbytes),
                }
            }
        }
    }
}

const FS6IPC_MESSAGE_SUCCESS: i32 = 1;
const WM_IPCTHREADACCESS: u32 = WM_USER + 130;
