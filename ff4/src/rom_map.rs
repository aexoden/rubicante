pub(crate) mod record {
    pub struct Record {
        pub address: usize,
        pub length: usize,
    }

    pub const GAME_TITLE: Record = Record {
        address: 0x00FFC0,
        length: 21,
    };

    pub const OUTDOOR_TILESET_PALETTE: Record = Record {
        address: 0x148900,
        length: 0x80,
    };

    pub const OUTDOOR_TILESET_COMPOSITION: Record = Record {
        address: 0x148000,
        length: 0x200,
    };

    pub const OUTDOOR_TILESET_UPPER_VALUES: Record = Record {
        address: 0x148600,
        length: 0x100,
    };

    pub const OUTDOOR_TILESET_LOWER_VALUES: Record = Record {
        address: 0x1D8000,
        length: 0x2000,
    };
}

const HASH_USA: &str = "680535dc1c4196c53b40dc9c2c9bc159a77802ab8d4b474bef5dc0281c15ad06";
const HASH_USA_REV_A: &str = "414bacc05a18a6137c0de060b4094ab6d1b75105342b0bb36a42e45d945a0e4d";

#[derive(Copy, Clone)]
pub enum Version {
    Us,
    UsRevA,
}

pub fn get_description(version: Version) -> String {
    match version {
        Version::Us => String::from("Final Fantasy II (USA)"),
        Version::UsRevA => String::from("Final Fantasy II (USA) (Rev A)"),
    }
}

pub fn get_version(hash: &str) -> Option<Version> {
    match hash {
        HASH_USA => Some(Version::Us),
        HASH_USA_REV_A => Some(Version::UsRevA),
        _ => None,
    }
}
