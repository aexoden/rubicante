use std::error::Error;
use std::fs;
use std::str;

use image::Rgba;
use sha2::{Digest, Sha256};

use super::rom_map;

pub struct Rom {
    data: Vec<u8>,
    version: rom_map::Version,
}

impl Rom {
    pub fn new(filename: &str) -> Result<Rom, Box<dyn Error>> {
        let mut data = fs::read(filename)?;

        remove_header_if_present(&mut data);

        let hash = hex::encode(Sha256::new().chain(&data).finalize());

        if let Some(version) = rom_map::get_version(&hash) {
            Ok(Rom { data, version })
        } else {
            Err("Unrecognized file".into())
        }
    }

    pub fn description(&self) -> String {
        rom_map::get_description(self.version)
    }

    pub fn title(&self) -> String {
        String::from_utf8_lossy(self.read_bytes(rom_map::record::GAME_TITLE, 0)).to_string()
    }

    pub(crate) fn read_bytes(&self, record: rom_map::record::Record, index: usize) -> &[u8] {
        let offset = address_to_rom_offset(record.address + record.length * index);
        &self.data[offset..offset + record.length]
    }

    pub(crate) fn read_palette(
        &self,
        record: rom_map::record::Record,
        index: usize,
        count: usize,
    ) -> Vec<Rgba<u8>> {
        (0..(count * record.length))
            .map(|i| {
                snes_color_to_rgba(self.read_u16(record.address + index * record.length + i * 2))
            })
            .collect()
    }

    pub fn read_u8(&self, address: usize) -> u8 {
        self.data[address_to_rom_offset(address)]
    }

    pub(crate) fn read_u16(&self, address: usize) -> u16 {
        self.read_u8(address) as u16 + ((self.read_u8(address + 1) as u16) << 8)
    }
}

fn address_to_rom_offset(address: usize) -> usize {
    let bank = address >> 16;
    let offset = (address & 0xFFFF) - 0x8000;

    (bank << 15) + offset
}

fn snes_color_to_rgba(color: u16) -> Rgba<u8> {
    let r = (color & 0x1F) as u8;
    let g = ((color >> 5) & 0x1F) as u8;
    let b = ((color >> 10) & 0x1F) as u8;

    Rgba([r * 8 + r / 4, g * 8 + g / 4, b * 8 + b / 4, 255])
}

fn remove_header_if_present(data: &mut Vec<u8>) {
    if data.len() % 1048576 == 512 {
        data.drain(..512);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_to_rom_offset() {
        assert_eq!(address_to_rom_offset(0x008000), 0x000000);
        assert_eq!(address_to_rom_offset(0x018000), 0x008000);
        assert_eq!(address_to_rom_offset(0x14FFFF), 0x0A7FFF);
        assert_eq!(address_to_rom_offset(0x158000), 0x0A8000);
        assert_eq!(address_to_rom_offset(0x15FFFF), 0x0AFFFF);
    }
}
