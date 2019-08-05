pub mod process_reader;
pub mod sa2_structures;
pub mod sa2_units;

use process_reader::ProcessHandle;

// Represents a structure that can be savestated.
pub trait SaveStateable {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str>;
    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str>;
}

// Nice type to do pointers for us.
pub struct Pointer<T>(T);

impl<T> Pointer<T> {
    pub fn new(inner: T) -> Pointer<T> {
        Pointer(inner)
    }
}

impl<T> SaveStateable for Pointer<T>
where
    T: SaveStateable,
{
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        let ptr_value = handle.read_u32(address)? as u64;
        if ptr_value != 0 {
            self.0.save(handle, ptr_value)
        } else {
            Err("reading null pointer")
        }
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        let ptr_value = handle.read_u32(address)? as u64;
        if ptr_value != 0 {
            self.0.load(handle, ptr_value)
        } else {
            Err("reading null pointer")
        }
    }
}

// impl for u8 for convenience
// Probably should have more types, too.
impl SaveStateable for u8 {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        *self = handle.read_u8(address)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &[*self])?;
        Ok(())
    }
}

// Represents a whole unit of stuff to save.
pub trait SaveStateUnit {
    fn save(&mut self, handle: &ProcessHandle) -> Result<(), &'static str>;
    fn load(&self, handle: &ProcessHandle) -> Result<(), &'static str>;
}
