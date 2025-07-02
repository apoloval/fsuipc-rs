
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

use user32::{FindWindowExA, SendMessageTimeoutA};
use winapi::WM_USER;
use winapi::windef::HWND;
use winapi::winuser::SMTO_BLOCK;

use super::{Handle, Session};
use super::ipc::*;
use super::raw::MutRawBytes;

/// A handle to FSUIPc that uses local IPC communication to the FSUIPC module
/// This kind of handle must be used from code running in the same process as FSUIPC does.
#[derive(Clone)]
pub struct LocalHandle {
    handle: HWND,
}

unsafe impl Send for LocalHandle {}

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
    #[cfg(target_pointer_width = "64")]
    destinations: Vec<*mut u8>,
}

impl LocalSession {
    fn new(handle: HWND) -> Self {
        let mut session = LocalSession {
            handle: handle,
            buffer: io::Cursor::new(Vec::with_capacity(4096)),
            #[cfg(target_pointer_width = "64")]
            destinations: Vec::new(),
        };
        // First 4-bytes seems to be for a stack frame pointer that is not actually used
        session.buffer.set_position(4);
        session
    }
}

impl Session for LocalSession {
    fn read_bytes(&mut self, offset: u16, dest: *mut u8, len: usize) -> io::Result<usize> {
        #[cfg(target_pointer_width = "64")]
        {
            let idx = self.destinations.len();
            self.destinations.push(dest);
            self.buffer.write_rsd(offset, idx as *mut u8, len)
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            self.buffer.write_rsd(offset, dest, len)
        }
    }

    fn write_bytes(&mut self, offset: u16, src: *const u8, len: usize) -> io::Result<usize> {
        self.buffer.write_wsd(offset, src, len)
    }

    fn process(mut self) -> io::Result<usize> {
        unsafe {
            self.buffer.write_header(&MsgHeader::TerminationMark)?;
            let nbytes = self.buffer.position() as usize;
            let buff = self.buffer.get_ref().as_ptr() as WinInt;
            let mut process_result: WinUInt = 0;
            let send_result = SendMessageTimeoutA(
                self.handle,
                WM_IPCTHREADACCESS,
                nbytes as WinUInt,
                buff,
                SMTO_BLOCK,
                WM_IPC_TIMEOUT,
                &mut process_result as *mut WinUInt);
            if send_result == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "timed out while waiting for a response from FSUIPC"));
            }
            if process_result != FS6IPC_MESSAGE_SUCCESS {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                    "FSUIPC rejected the requests with error {}; possible buffer corruption in bytes: {:?}",
                    send_result, self.buffer.get_ref())));
            }
            // First 4-bytes seems to be for a stack frame pointer that is not actually used
            self.buffer.set_position(4);
            loop {
                let header = self.buffer.read_header()?;
                match &header {
                    &MsgHeader::ReadStateData { offset: _, len, target } => {
                        #[cfg(target_pointer_width = "64")]
                        let actual = {
                            let idx = target as usize;
                            *self.destinations.get(idx).ok_or_else(|| {
                                io::Error::new(io::ErrorKind::InvalidData, "invalid destination index")
                            })?
                        };
                        #[cfg(target_pointer_width = "64")]
                        let mut output = MutRawBytes::new(actual, len);
                        #[cfg(not(target_pointer_width = "64"))]
                        let mut output = MutRawBytes::new(target, len);
                        self.buffer.read_body(&header, &mut output)?;
                    },
                    &MsgHeader::WriteStateData { offset: _, len: _ } => {
                        let mut output = io::sink();
                        self.buffer.read_body(&header, &mut output)?;
                    },
                    &MsgHeader::TerminationMark => return Ok(nbytes),
                }
            }
        }
    }
}

const FS6IPC_MESSAGE_SUCCESS: WinUInt = 1;
const WM_IPCTHREADACCESS: u32 = WM_USER + 130;
const WM_IPC_TIMEOUT: u32 = 10000;

#[cfg(test)]
mod test {
    use std::thread;
    use winapi::windef::HWND;
    use super::*;

    #[test]
    fn test_local_handler_can_be_shared() {
        let handler = LocalHandle{ handle: 0 as HWND };
        let handler_copy = handler.clone();
        let child = thread::spawn(move|| {
            assert_eq!(0 as HWND, handler_copy.handle);
        });
        child.join().unwrap();
        assert_eq!(0 as HWND, handler.handle);
    }
}