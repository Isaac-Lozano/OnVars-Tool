use std::mem;
use std::ffi::CStr;
use std::vec::IntoIter;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{HMODULE, MAX_PATH};
use winapi::shared::ntdef::NULL;
use winapi::um::memoryapi;
use winapi::um::processthreadsapi;
use winapi::um::psapi;
use winapi::um::winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, PROCESS_VM_WRITE};

const PROCESS_BUFFER_LEN: usize = 1024;

#[derive(Clone,Copy,Debug)]
pub struct ProcessHandle(HANDLE);

impl ProcessHandle {
    fn open_process(id: ProcessId, mode: u32) -> Result<ProcessHandle, &'static str> {
        let handle;
        unsafe {
            handle = processthreadsapi::OpenProcess(mode, false as i32, id.0);
            if handle == NULL {
                return Err("could not open process");
            }
        }
        Ok(ProcessHandle(handle))
    }

    pub fn open_process_read_info(id: ProcessId) -> Result<ProcessHandle, &'static str> {
        Self::open_process(id, PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE)
    }

    pub fn get_name(&self) -> Result<String, &'static str> {
        let name;
        unsafe {
            let mut module = mem::uninitialized();
            let mut bytes_needed = mem::uninitialized();
            let result = psapi::EnumProcessModules(self.0, &mut module as *mut HMODULE, mem::size_of::<HMODULE>() as u32, &mut bytes_needed as *mut u32);
            if result == 0 {
                return Err("error in EnumProcessModules");
            }
            let mut name_buffer = [0i8; MAX_PATH];
            let bytes_in_str = psapi::GetModuleBaseNameA(self.0, module, &mut name_buffer[0] as *mut i8, MAX_PATH as u32);
            let name_buffer: [u8; MAX_PATH] = mem::transmute(name_buffer);
            name = CStr::from_bytes_with_nul(&name_buffer[.. bytes_in_str as usize + 1])
                .map_err(|_| "error converting process name")?
                .to_str()
                .map_err(|_| "error converting process name")?
                .to_string();
        }
        Ok(name)
    }

    pub fn from_name_filter<F>(mut filter: F) -> Result<Option<ProcessHandle>, &'static str>
        where F: FnMut(String) -> bool,
    {
        let mut processes = ProcessIterator::new()?
            .filter_map(|pid| {
                let handle = ProcessHandle::open_process_read_info(pid).ok()?;
                let name = handle.get_name().ok()?;
                if filter(name) {
                    Some(handle)
                }
                else {
                    None
                }
            });
        Ok(processes.next())
    }

    pub fn read_data(&self, address: u64, buf: &mut [u8]) -> Result<usize, &'static str> {
        let mut bytes_read;
        unsafe {
            bytes_read = mem::uninitialized();
            let address = mem::transmute(address);
            let buf_addr = buf.as_mut_ptr() as *mut c_void;
            let result = memoryapi::ReadProcessMemory(self.0, address, buf_addr, buf.len(), &mut bytes_read as *mut usize);
            if result == 0 {
                return Err("Error in ReadProcessMemory");
            }
        }
        Ok(bytes_read)
    }

    pub fn read_u8(&self, address: u64) -> Result<u8, &'static str> {
        let mut buf = [0; 1];
        let bytes_read = self.read_data(address, &mut buf)?;
        if bytes_read != 1 {
            panic!("Not enough bytes read");
        }
        Ok(buf[0])
    }

    pub fn read_i32(&self, address: u64) -> Result<i32, &'static str> {
        let mut buf = [0; 4];
        let bytes_read = self.read_data(address, &mut buf)?;
        if bytes_read != 4 {
            panic!("Not enough bytes read");
        }
        let value = i32::from_le_bytes(buf);
        Ok(value)
    }

    pub fn read_u32(&self, address: u64) -> Result<u32, &'static str> {
        let mut buf = [0; 4];
        let bytes_read = self.read_data(address, &mut buf)?;
        if bytes_read != 4 {
            panic!("Not enough bytes read");
        }
        let value = u32::from_le_bytes(buf);
        Ok(value)
    }

    pub fn write_data(&self, address: u64, buf: &[u8]) -> Result<usize, &'static str> {
        let mut bytes_written;
        unsafe {
            bytes_written = mem::uninitialized();
            let address = mem::transmute(address);
            let buf_addr = buf.as_ptr() as *const c_void;
            let result = memoryapi::WriteProcessMemory(self.0, address, buf_addr, buf.len(), &mut bytes_written as *mut usize);
            if result == 0 {
                return Err("Error in WriteProcessMemory");
            }
        }
        Ok(bytes_written)
    }

    pub fn write_u32(&self, address: u64, value: u32) -> Result<(), &'static str> {
        let buf = value.to_le_bytes();
        let bytes_written = self.write_data(address, &buf)?;
        if bytes_written == 4 {
            Ok(())
        } else {
            Err("not enough bytes written")
        }
    }

    pub fn write_u8(&self, address: u64, value: u8) -> Result<(), &'static str> {
        let buf = [value];
        let bytes_written = self.write_data(address, &buf)?;
        if bytes_written == 1 {
            Ok(())
        } else {
            Err("not enough bytes written")
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub struct ProcessId(u32);

#[derive(Clone,Debug)]
pub struct ProcessIterator {
    iter: IntoIter<u32>,
}

impl ProcessIterator {
    pub fn new() -> Result<ProcessIterator, &'static str> {
        let mut buffer = vec![0; PROCESS_BUFFER_LEN];

        unsafe {
            let buf_ptr = buffer.as_mut_ptr();
            let mut returned_bytes = 0u32;
            let result = psapi::EnumProcesses(buf_ptr, (PROCESS_BUFFER_LEN * mem::size_of::<u32>()) as u32, &mut returned_bytes as *mut u32);
            if result == 0 {
                return Err("Error in EnumProcess");
            }
            buffer.set_len(returned_bytes as usize / mem::size_of::<u32>());
        }

        Ok(ProcessIterator {
            iter: buffer.into_iter(),
        })
    }
}

impl Iterator for ProcessIterator {
    type Item = ProcessId;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(ProcessId)
    }
}
