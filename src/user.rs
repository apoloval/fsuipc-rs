//
// FSUIPC library
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::CString;
use std::io;
use std::os::raw::c_void;
use std::ptr;

use kernel32::*;
use user32::{FindWindowExA, RegisterWindowMessageA, SendMessageA};
use winapi::memoryapi::FILE_MAP_WRITE;
use winapi::minwindef::ATOM;
use winapi::windef::HWND;
use winapi::winnt::{HANDLE, PAGE_READWRITE};

use super::{Handle, Session};
use super::ipc::*;
use super::raw::{MutRawBytes, RawBytes};

pub struct UserHandle {
    handle: HWND,
    file_mapping_atom: ATOM,
    file_mapping: HANDLE,
    msg_id: u32,
    data: *mut u8,
}

impl UserHandle {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let win_name = CString::new("UIPCMAIN").unwrap();
            let handle = FindWindowExA(
                ptr::null_mut(),
                ptr::null_mut(),
                win_name.as_ptr(),
                ptr::null_mut());
            if handle == ptr::null_mut() {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to user FSUIPC: cannot create window handle"));
            }
            let msg_name = CString::new("FsasmLib:IPC").unwrap();
            let msg_id = RegisterWindowMessageA(msg_name.as_ptr());
            if msg_id == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to user FSUIPC: cannot register window message"));
            }

            let file_mapping_name = CString::new(
                format!("FsasmLib:IPC:{:x}:{:x}",
                    GetCurrentProcessId(),
                    next_file_mapping_index())).unwrap();

            let file_mapping_atom = GlobalAddAtomA(file_mapping_name.as_ptr());
            if file_mapping_atom == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to user FSUIPC: cannot add global atom"));
            }

            let file_mapping = CreateFileMappingA(
                0xffffffff as HANDLE,
                ptr::null_mut(),
                PAGE_READWRITE,
                0, FILE_MAPPING_LEN as u32,
                file_mapping_name.as_ptr());
            if file_mapping == ptr::null_mut() {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to user FSUIPC: cannot create file mapping"));
            }
            let data = MapViewOfFile(file_mapping, FILE_MAP_WRITE, 0, 0, 0) as *mut u8;
            if data == ptr::null_mut() {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to user FSUIPC: cannot map view of file"));
            }
            Ok(UserHandle {
                handle: handle,
                file_mapping_atom: file_mapping_atom,
                file_mapping: file_mapping,
                msg_id: msg_id,
                data: data,
            })
        }
    }
}

impl Handle for UserHandle {
    type Sess = UserSession;

    fn session(&self) -> UserSession {
        UserSession {
            handle: self.handle,
            file_mapping_atom: self.file_mapping_atom,
            msg_id: self.msg_id,
            data: self.data,
            buffer: MutRawBytes::new(self.data, FILE_MAPPING_LEN)
        }
    }
}

impl Drop for UserHandle {
    fn drop(&mut self) {
        unsafe {
            GlobalDeleteAtom(self.file_mapping_atom);
            UnmapViewOfFile(self.data as *const c_void);
            CloseHandle(self.file_mapping);
        }
    }
}

pub struct UserSession {
    handle: HWND,
    file_mapping_atom: ATOM,
    msg_id: u32,
    data: *mut u8,
    buffer: MutRawBytes,
}

impl Session for UserSession {
    fn read_bytes(&mut self, offset: u16, dest: *mut u8, len: usize) -> io::Result<usize> {
        self.buffer.write_rsd(offset, dest, len)
    }

    fn write_bytes(&mut self, offset: u16, src: *const u8, len: usize) -> io::Result<usize> {
        self.buffer.write_wsd(offset, src, len)
    }

    fn process(mut self) -> io::Result<usize> {
        unsafe {
            try!(self.buffer.write_header(&MsgHeader::TerminationMark));
            let send_result = SendMessageA(
                self.handle,
                self.msg_id,
                self.file_mapping_atom as u32,
                0);
            if send_result != FS6IPC_MESSAGE_SUCCESS {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                    "FSUIPC rejected the requests with error {}; possible buffer corruption",
                    send_result)));
            }
            let mut buffer = RawBytes::new(self.data, FILE_MAPPING_LEN);
            loop {
                let header = try!(buffer.read_header());
                match &header {
                    &MsgHeader::ReadStateData { offset: _, len, target } => {
                        let mut output = MutRawBytes::new(target, len);
                        try!(buffer.read_body(&header, &mut output));
                    },
                    &MsgHeader::WriteStateData { offset: _, len: _ } => {
                        let mut output = io::sink();
                        try!(buffer.read_body(&header, &mut output));
                    },
                    &MsgHeader::TerminationMark => return Ok(buffer.consumed()),
                }
            }
        }
    }
}

fn next_file_mapping_index() -> u32 {
    unsafe {
        let next = FILE_MAPPING_INDEX;
        FILE_MAPPING_INDEX += 1;
        next
    }
}

const FS6IPC_MESSAGE_SUCCESS: i32 = 1;
const FILE_MAPPING_LEN: usize = 64*1024;

static mut FILE_MAPPING_INDEX: u32 = 0;
