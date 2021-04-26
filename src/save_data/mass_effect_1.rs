use anyhow::Result;
use serde::{ser::SerializeStruct, Serialize};
use std::io::{Cursor, Read, Write};
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{gui::Gui, save_data::Dummy, unreal};

use super::{List, SaveCursor, SaveData};

pub mod player;
use self::player::*;

pub mod state;
use self::state::*;

pub mod data;
pub mod known_plot;

#[derive(Clone)]
pub struct Me1SaveGame {
    _begin: Dummy<8>,
    zip_offset: u32,
    _no_mans_land: List<u8>,
    pub player: Player,
    pub state: State,
    _world_save_package: Option<WorldSavePackage>,
}

impl Me1SaveGame {
    fn zip(&self) -> Result<List<u8>> {
        let mut zip = Vec::new();
        {
            let mut zipper = ZipWriter::new(Cursor::new(&mut zip));
            let options = FileOptions::default().compression_method(CompressionMethod::DEFLATE);

            // Player
            {
                let player_data = unreal::Serializer::to_bytes(&self.player)?;
                zipper.start_file("player.sav", options)?;
                zipper.write_all(&player_data)?;
            }
            // State
            {
                let state_data = unreal::Serializer::to_bytes(&self.state)?;
                zipper.start_file("state.sav", options)?;
                zipper.write_all(&state_data)?;
            }
            // WorldSavePackage
            if let Some(_world_save_package) = &self._world_save_package {
                let world_save_package_data = unreal::Serializer::to_bytes(_world_save_package)?;
                zipper.start_file("WorldSavePackage.sav", options)?;
                zipper.write_all(&world_save_package_data)?;
            }
        }
        Ok(zip.into())
    }
}

impl SaveData for Me1SaveGame {
    fn deserialize(cursor: &mut SaveCursor) -> Result<Self> {
        let _begin: Dummy<8> = SaveData::deserialize(cursor)?;
        let zip_offset: u32 = SaveData::deserialize(cursor)?;
        let _no_mans_land = cursor.read(zip_offset as usize - 12)?.into();

        let zip_data = Cursor::new(cursor.read_to_end()?);
        let mut zip = ZipArchive::new(zip_data)?;

        let player: Player = {
            let mut bytes = Vec::new();
            zip.by_name("player.sav")?.read_to_end(&mut bytes)?;
            let mut cursor = SaveCursor::new(bytes);
            SaveData::deserialize(&mut cursor)?
        };

        let state: State = {
            let mut bytes = Vec::new();
            zip.by_name("state.sav")?.read_to_end(&mut bytes)?;
            let mut cursor = SaveCursor::new(bytes);
            SaveData::deserialize(&mut cursor)?
        };

        let _world_save_package: Option<WorldSavePackage> =
            if zip.file_names().any(|f| f == "WorldSavePackage.sav") {
                Some({
                    let mut bytes = Vec::new();
                    zip.by_name("WorldSavePackage.sav")?.read_to_end(&mut bytes)?;
                    let mut cursor = SaveCursor::new(bytes);
                    SaveData::deserialize(&mut cursor)?
                })
            } else {
                None
            };

        Ok(Self { _begin, zip_offset, _no_mans_land, player, state, _world_save_package })
    }

    fn draw_raw_ui(&mut self, _: &Gui, _: &str) {}
}

impl serde::Serialize for Me1SaveGame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        let Me1SaveGame {
            _begin,
            zip_offset,
            _no_mans_land,
            player: _,
            state: _,
            _world_save_package,
        } = self;

        let mut s = serializer.serialize_struct("Me1SaveGame", 4)?;
        s.serialize_field("_begin", _begin)?;
        s.serialize_field("zip_offset", zip_offset)?;
        s.serialize_field("_no_mans_land", _no_mans_land)?;
        s.serialize_field("zip", &self.zip().map_err(Error::custom)?)?;
        s.end()
    }
}

#[derive(Serialize, Clone)]
pub(super) struct WorldSavePackage {
    data: List<u8>,
}

impl SaveData for WorldSavePackage {
    fn deserialize(cursor: &mut SaveCursor) -> Result<Self> {
        Ok(Self { data: cursor.read_to_end()?.into() })
    }

    fn draw_raw_ui(&mut self, _: &Gui, _: &str) {}
}

#[cfg(test)]
mod test {
    use anyhow::*;
    use std::{
        time::Instant,
        {fs::File, io::Read},
    };

    use crate::{save_data::*, unreal};

    use super::*;

    #[test]
    fn unzip_deserialize_serialize_zip() -> Result<()> {
        let files = [
            "test/Clare00_AutoSave.MassEffectSave", // Avec WorldSavePackage.sav
            "test/Char_01-60-3-2-2-26-6-2018-57-26.MassEffectSave", // Sans
        ];

        for file in &files {
            let mut input = Vec::new();
            {
                let mut file = File::open(file)?;
                file.read_to_end(&mut input)?;
            }

            let now = Instant::now();

            // Deserialize
            let mut cursor = SaveCursor::new(input);
            let me1_save_game = Me1SaveGame::deserialize(&mut cursor)?;

            println!("Deserialize 1 : {:?}", Instant::now().saturating_duration_since(now));
            let now = Instant::now();

            // Serialize
            let output = unreal::Serializer::to_bytes(&me1_save_game)?;

            println!("Serialize 1 : {:?}", Instant::now().saturating_duration_since(now));
            let now = Instant::now();

            // Deserialize (again)
            let mut cursor = SaveCursor::new(output.clone());
            let me1_save_game = Me1SaveGame::deserialize(&mut cursor)?;

            println!("Deserialize 2 : {:?}", Instant::now().saturating_duration_since(now));
            let now = Instant::now();

            // Serialize (again)
            let output_2 = unreal::Serializer::to_bytes(&me1_save_game)?;

            println!("Serialize 2 : {:?}", Instant::now().saturating_duration_since(now));

            // Check 2nd serialize = first serialize
            let cmp = output.chunks(4).zip(output_2.chunks(4));
            for (i, (a, b)) in cmp.enumerate() {
                if a != b {
                    panic!("0x{:02x?} : {:02x?} != {:02x?}", i * 4, a, b);
                }
            }
        }
        Ok(())
    }
}
