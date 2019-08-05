use crate::process_reader::ProcessHandle;
use crate::{SaveStateable, Pointer};

struct CollisionElement([u8;0x30]);

impl CollisionElement {
    fn new() -> CollisionElement {
        CollisionElement([0;0x30])
    }
}

impl SaveStateable for CollisionElement {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.read_data(address, &mut self.0)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &self.0)?;
        Ok(())
    }
}

struct CollisionData {
    data: [u8;0xa8],
    element_array: Pointer<CollisionElement>,
}

impl CollisionData {
    fn new() -> CollisionData {
        CollisionData {
            data: [0;0xa8],
            element_array: Pointer::new(CollisionElement::new()),
        }
    }
}

impl SaveStateable for CollisionData {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.read_data(address, &mut self.data)?;
        self.element_array.save(handle, address + 0xc)
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &self.data)?;
        self.element_array.load(handle, address + 0xc)
    }
}

// ActionStruct: 0x30 bytes
// This also has some data in the collision stuff that we're interested as well.
// This is why we have another 0x30 bytes we're saving.
struct ActionStruct{
    data: [u8;0x30],
    collision_data: Pointer<CollisionData>,
}

impl ActionStruct {
    fn new() -> ActionStruct {
        ActionStruct {
            data: [0;0x30],
            collision_data: Pointer::new(CollisionData::new()),
        }
    }
}

impl SaveStateable for ActionStruct {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.read_data(address, &mut self.data)?;
        self.collision_data.save(handle, address + 0x2c)
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &self.data)?;
        self.collision_data.load(handle, address + 0x2c)
    }
}

// GlobalMetricStruct: 0x40 bytes
struct GlobalMetricStruct([u8;0x40]);

impl SaveStateable for GlobalMetricStruct {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.read_data(address, &mut self.0)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &self.0)?;
        Ok(())
    }
}

// PhysicsStruct: Variable based on character
enum CharacterPhys {
    SpeedPhys([u8;0x3a0]),
    HuntPhys([u8;0x420]),
    MechPhys([u8;0x454]),
}

impl SaveStateable for CharacterPhys {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        let character_id = handle.read_u8(address + 0x1)?;
        match character_id {
            0 | 1 => {
                let mut buf = [0;0x3a0];
                handle.read_data(address, &mut buf)?;
                *self = CharacterPhys::SpeedPhys(buf);
                Ok(())
            }
            4 | 5 => {
                let mut buf = [0;0x420];
                handle.read_data(address, &mut buf)?;
                *self = CharacterPhys::HuntPhys(buf);
                Ok(())
            }
            6 | 7 => {
                let mut buf = [0;0x454];
                handle.read_data(address, &mut buf)?;
                *self = CharacterPhys::MechPhys(buf);
                Ok(())
            }
            // Doesn't handle Super Sonic and Mechless, yet.
            _ => Err("character type not supported"),
        }
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        let character_id = handle.read_u8(address + 0x1)?;
        match *self {
            CharacterPhys::SpeedPhys(buf) => {
                if character_id != 0 && character_id != 1 {
                    return Err("current character does not match savestate character");
                }
                handle.write_data(address, &buf)?;
            }
            CharacterPhys::HuntPhys(buf) => {
                if character_id != 4 && character_id != 5 {
                    return Err("current character does not match savestate character");
                }
                handle.write_data(address, &buf)?;
            }
            CharacterPhys::MechPhys(buf) => {
                if character_id != 6 && character_id != 7 {
                    return Err("current character does not match savestate character");
                }
                handle.write_data(address, &buf)?;
            }
        }
        Ok(())
    }
}

// Struct that holds info about collision with the level
struct LevelCollision([u8;0x84]);

impl LevelCollision {
    fn new() -> LevelCollision {
        LevelCollision([0;0x84])
    }
}

impl SaveStateable for LevelCollision {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.read_data(address, &mut self.0)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &self.0)?;
        Ok(())
    }
}

// Top level physics struct
struct PhysicsStruct {
    data: CharacterPhys,
    level_collision: Pointer<LevelCollision>,
}

impl PhysicsStruct {
    fn new() -> PhysicsStruct {
        PhysicsStruct {
            data: CharacterPhys::SpeedPhys([0;0x3a0]),
            level_collision: Pointer::new(LevelCollision::new()),
        }
    }
}

impl SaveStateable for PhysicsStruct {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        self.data.save(handle, address)?;
        self.level_collision.save(handle, address + 0x90)
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        self.data.load(handle, address)?;
        self.level_collision.load(handle, address + 0x90)
    }
}

// Character Task Struct
// We don't care about all the funciton pointers. They don't change.
// We just care about the pointers to data.
pub struct Character {
    acs: Pointer<ActionStruct>,
    gms: Pointer<GlobalMetricStruct>,
    phs: Pointer<PhysicsStruct>,
}

impl Character {
    pub fn new() -> Character {
        Character {
            acs: Pointer(ActionStruct::new()),
            gms: Pointer(GlobalMetricStruct([0;0x40])),
            phs: Pointer(PhysicsStruct::new()),
        }
    }
}

impl SaveStateable for Character {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        self.acs.save(handle, address + 0x34)?;
        self.gms.save(handle, address + 0x38)?;
        self.phs.save(handle, address + 0x40)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        self.acs.load(handle, address + 0x34)?;
        self.gms.load(handle, address + 0x38)?;
        self.phs.load(handle, address + 0x40)?;
        Ok(())
    }
}

// Array of 4 at 0x01dcff40
// Size of each element is 0x24d8
// We only save the first element
// We also save a size 0x40 set of data at 0x01dcff00
// These are contiguous, so might as well read it all together
pub struct Camera([u8;0x2518]);

impl Camera {
    pub fn new() -> Camera {
        Camera([0;0x2518])
    }
}

impl SaveStateable for Camera {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.read_data(address, &mut self.0)?;
        // These values may potentially be needed, but don't seem to affect savestates.
//        handle.read_data(0x019f3190, &mut self.1)?;
//        handle.read_data(0x019f31d0, &mut self.2)?;
//        handle.read_data(0x019f317c, &mut self.3)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &self.0)?;
//        handle.write_data(0x019f3190, &self.1)?;
//        handle.write_data(0x019f31d0, &self.2)?;
//        handle.write_data(0x019f317c, &self.3)?;
        Ok(())
    }
}

// PastPositionTable: 0xc000 bytes
// A set of 0x100 3vecs.
pub struct PastPositionTable([u8;0xc00]);

impl PastPositionTable {
    pub fn new() -> PastPositionTable {
        PastPositionTable([0;0xc00])
    }
}

impl SaveStateable for PastPositionTable {
    fn save(&mut self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.read_data(address, &mut self.0)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle, address: u64) -> Result<(), &'static str> {
        handle.write_data(address, &self.0)?;
        Ok(())
    }
}
