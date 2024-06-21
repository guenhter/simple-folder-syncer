use std::fs::File;
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::os::windows::io::FromRawHandle;
use std::path::Path;
use std::ptr;
use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
use windows_sys::Wdk::Storage::FileSystem::{NtCreateFile, FILE_OPEN, FILE_OPEN_REPARSE_POINT};
use windows_sys::Win32::Foundation::{
    RtlNtStatusToDosError, ERROR_DELETE_PENDING, STATUS_DELETE_PENDING, STATUS_PENDING,
    UNICODE_STRING,
};
use windows_sys::Win32::Storage::FileSystem::{
    FileDispositionInfoEx, SetFileInformationByHandle, DELETE, FILE_DISPOSITION_FLAG_DELETE,
    FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE, FILE_DISPOSITION_FLAG_POSIX_SEMANTICS,
    FILE_DISPOSITION_INFO_EX, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_LIST_DIRECTORY, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, SYNCHRONIZE,
};
use windows_sys::Win32::System::Kernel::OBJ_DONT_REPARSE;
use windows_sys::Win32::System::IO::{IO_STATUS_BLOCK, IO_STATUS_BLOCK_0};

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn main() {
    let path = r"Z:\latest\.xxx";
    let subpath = "y";
    let filename = "some.txt";

    let dir = open_link(path.as_ref(), DELETE | FILE_LIST_DIRECTORY).unwrap();
    let subdir = open_link_no_reparse(
        &dir,
        &utf16(subpath),
        SYNCHRONIZE | DELETE | FILE_LIST_DIRECTORY,
    )
    .unwrap();
    let f = open_link_no_reparse(&subdir, &utf16(filename), SYNCHRONIZE | DELETE).unwrap();
    f.posix_delete().unwrap();
}

trait FileExt {
    fn posix_delete(&self) -> io::Result<()>;
}
impl FileExt for File {
    fn posix_delete(&self) -> io::Result<()> {
        unsafe {
            let handle = self.as_raw_handle() as _;
            let info = FILE_DISPOSITION_INFO_EX {
                Flags: FILE_DISPOSITION_FLAG_DELETE
                    | FILE_DISPOSITION_FLAG_POSIX_SEMANTICS
                    | FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE,
            };
            let result = SetFileInformationByHandle(
                handle,
                FileDispositionInfoEx,
                std::ptr::from_ref(&info).cast(),
                std::mem::size_of::<FILE_DISPOSITION_INFO_EX>() as u32,
            );
            if result == 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }
}

fn open_link(path: &Path, access_mode: u32) -> io::Result<File> {
    File::options()
        .access_mode(access_mode)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

fn open_link_no_reparse(parent: &File, name: &[u16], access: u32) -> io::Result<File> {
    unsafe {
        let mut handle = 0;
        let mut io_status = IO_STATUS_BLOCK {
            Anonymous: IO_STATUS_BLOCK_0 {
                Status: STATUS_PENDING,
            },
            Information: 0,
        };
        let mut name_str = UNICODE_STRING {
            Length: (name.len() * 2) as u16,
            MaximumLength: (name.len() * 2) as u16,
            Buffer: name.as_ptr().cast_mut(),
        };
        let object = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            ObjectName: &mut name_str,
            RootDirectory: parent.as_raw_handle() as _,
            Attributes: OBJ_DONT_REPARSE as _,
            SecurityDescriptor: ptr::null(),
            SecurityQualityOfService: ptr::null(),
        };
        let status = NtCreateFile(
            &mut handle,
            access,
            &object,
            &mut io_status,
            ptr::null_mut(),
            0,
            FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_OPEN,
            FILE_OPEN_REPARSE_POINT,
            ptr::null_mut(),
            0,
        );
        eprintln!("NtCreateFile: {status:#X}");
        if status >= 0 {
            Ok(File::from_raw_handle(handle as _))
        } else if status == STATUS_DELETE_PENDING {
            Err(io::Error::from_raw_os_error(ERROR_DELETE_PENDING as i32))
        } else {
            Err(io::Error::from_raw_os_error(
                RtlNtStatusToDosError(status) as _
            ))
        }
    }
}
