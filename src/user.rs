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

use kernel32::{CreateFileMappingA, GetCurrentProcessId, MapViewOfFile};
use user32::{FindWindowExA, RegisterWindowMessageA};
use winapi::memoryapi::FILE_MAP_WRITE;
use winapi::windef::HWND;
use winapi::winnt::{HANDLE, PAGE_READWRITE};

use super::{Handle, Session};
use super::raw::MutRawBytes;

pub struct UserHandle {
    handle: HWND,
    file_mapping: HANDLE,
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
            let msg = RegisterWindowMessageA(msg_name.as_ptr());
            if msg == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "cannot connect to user FSUIPC: cannot register window message"));
            }
            let file_mapping_name = CString::new(
                format!("FsasmLib:IPC:{:x}:{:x}",
                    GetCurrentProcessId(),
                    next_file_mapping_index())).unwrap();
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
                file_mapping: file_mapping,
                data: data,
            })
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

static FILE_MAPPING_LEN: usize = 64*1024;
static mut FILE_MAPPING_INDEX: u32 = 0;
