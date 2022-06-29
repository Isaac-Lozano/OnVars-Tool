use crate::{SaveStateUnit, SaveStateable, Pointer};
use crate::process_reader::ProcessHandle;
use crate::sa2_structures::{Character, Camera, PastPositionTable};


pub struct CharacterUnit {
    character: Pointer<Character>,
}

impl CharacterUnit {
    pub fn new() -> CharacterUnit {
        CharacterUnit {
            character: Pointer::new(Character::new()),
        }
    }
}

impl SaveStateUnit for CharacterUnit {
    fn save(&mut self, handle: &ProcessHandle) -> Result<(), &'static str> {

        self.character.save(handle, 0x01dea6e0)
    }

    fn load(&self, handle: &ProcessHandle) -> Result<(), &'static str> {
        
        self.character.load(handle, 0x01dea6e0)
    }
}

pub struct CameraUnit {
    camera: Camera,
    past_positions: Pointer<PastPositionTable>,
    past_rotations: Pointer<PastPositionTable>,
    past_positions_idx: u8,
    camera_cons_past_positions: PastPositionTable,
    camera_cons_past_positions_idx: u8,
}

impl CameraUnit {
    pub fn new() -> CameraUnit {
        CameraUnit {
            camera: Camera::new(),
            past_positions: Pointer::new(PastPositionTable::new()),
            past_rotations: Pointer::new(PastPositionTable::new()),
            past_positions_idx: 0,
            camera_cons_past_positions: PastPositionTable::new(),
            camera_cons_past_positions_idx: 0,
        }
    }
}

impl SaveStateUnit for CameraUnit {
    fn save(&mut self, handle: &ProcessHandle) -> Result<(), &'static str> {
        self.camera.save(handle, 0x01dcff00)?;
        self.past_positions.save(handle, 0x01a5a234)?;
        self.past_rotations.save(handle, 0x01a5a238)?;
        self.past_positions_idx.save(handle, 0x01945910)?;
        self.camera_cons_past_positions.save(handle, 0x019f1740)?;
        self.camera_cons_past_positions_idx.save(handle, 0x019f173c)
    }

    fn load(&self, handle: &ProcessHandle) -> Result<(), &'static str> {
        self.camera.load(handle, 0x01dcff00)?;
        self.past_positions.load(handle, 0x01a5a234)?;
        self.past_rotations.load(handle, 0x01a5a238)?;
        self.past_positions_idx.load(handle, 0x01945910)?;
        self.camera_cons_past_positions.load(handle, 0x019f1740)?;
        self.camera_cons_past_positions_idx.load(handle, 0x019f173c)
    }
}

pub struct TimeUnit([u8;0x3]);

impl TimeUnit {
    pub fn new() -> TimeUnit {
        TimeUnit([0;0x3])
    }
}

impl SaveStateUnit for TimeUnit {
    fn save(&mut self, handle: &ProcessHandle) -> Result<(), &'static str> {
        handle.read_data(0x0174AFDB, &mut self.0)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle) -> Result<(), &'static str> {
        handle.write_data(0x0174AFDB, &self.0)?;
        Ok(())
    }
}

pub struct GravityUnit([u8;0xc]);

impl GravityUnit {
    pub fn new() -> GravityUnit {
        GravityUnit([0;0xc])
    }
}

impl SaveStateUnit for GravityUnit {
    fn save(&mut self, handle: &ProcessHandle) -> Result<(), &'static str> {
        handle.read_data(0x01DE94A0, &mut self.0)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle) -> Result<(), &'static str> {
        handle.write_data(0x01DE94A0, &self.0)?;
        Ok(())
    }
}

// Crashes the game. :(
pub struct LevelCollisionUnit([u8;0x3000], [u8;0x2]);

impl LevelCollisionUnit {
    pub fn new() -> LevelCollisionUnit {
        LevelCollisionUnit([0;0x3000], [0;0x2])
    }
}

impl SaveStateUnit for LevelCollisionUnit {
    fn save(&mut self, handle: &ProcessHandle) -> Result<(), &'static str> {
        handle.read_data(0x01a5a2dc, &mut self.0)?;
        handle.read_data(0x01de9484, &mut self.1)?;
        Ok(())
    }

    fn load(&self, handle: &ProcessHandle) -> Result<(), &'static str> {
        handle.write_data(0x01a5a2dc, &self.0)?;
        handle.write_data(0x01de9484, &self.1)?;
        Ok(())
    }
}
