use std::error::Error;
use std::fs;
use std::str;

mod rom_map;

pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn new(filename: &str) -> Result<Rom, Box<dyn Error>> {
        let mut data = fs::read(filename)?;

        if !validate_rom_size(&mut data) {
            return Err("Invalid file size".into());
        }

        Ok(Rom { data })
    }

    pub fn title(&self) -> String {
        String::from_utf8_lossy(
            self.read_bytes(rom_map::GAME_TITLE_ADDRESS, rom_map::GAME_TITLE_LENGTH),
        )
        .to_string()
    }

    fn read_bytes(&self, address: usize, bytes: usize) -> &[u8] {
        let offset = address_to_rom_offset(address);
        &self.data[offset..offset + bytes]
    }
}

fn address_to_rom_offset(address: usize) -> usize {
    let bank = address >> 16;
    let offset = (address & 0xFFFF) - 0x8000;

    (bank << 15) + offset
}

fn validate_rom_size(data: &mut Vec<u8>) -> bool {
    match data.len() {
        1048576 | 2097152 => true,
        1049088 | 2097664 => {
            data.drain(..512);
            true
        }
        _ => false,
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
