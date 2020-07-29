pub(crate) mod record {
    #[derive(Copy, Clone)]
    pub struct Record {
        pub address: usize,
        pub length: usize,
    }

    pub const CHARACTER_STATS_INITIAL: Record = Record {
        address: 0x0FA900,
        length: 0x20,
    };

    pub const OCEAN_ANIMATION_SEQUENCE: Record = Record {
        address: 0x008E8C,
        length: 0x10,
    };

    pub const WATERFALL_ANIMATION_SEQUENCE: Record = Record {
        address: 0x008E7C,
        length: 0x10,
    };

    pub const GAME_TITLE: Record = Record {
        address: 0x00FFC0,
        length: 21,
    };

    pub const OUTDOOR_TILEMAP_OVERWORLD: Record = Record {
        address: 0x168480,
        length: 0x4000,
    };

    pub const OUTDOOR_TILEMAP_UNDERWORLD: Record = Record {
        address: 0x16C480,
        length: 0x1D00,
    };

    pub const OUTDOOR_TILEMAP_MOON: Record = Record {
        address: 0x16E180,
        length: 0xA00,
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

    pub const FIELD_SPRITE_PALETTE_INDEX_PLAYER: Record = Record {
        address: 0x15B2FA,
        length: 0x01,
    };

    pub const FIELD_SPRITE_PALETTE_PLAYER: Record = Record {
        address: 0x0D8000,
        length: 0x10,
    };

    pub const FIELD_SPRITE_SHEET_PLAYER: Record = Record {
        address: 0x1B8000,
        length: 0x0300,
    };

    pub const FIELD_SPRITE_COMPOSITION_PLAYER: Record = Record {
        address: 0x15C0C4,
        length: 0x08,
    };

    pub const TITLE_TILES: Record = Record {
        address: 0x08C000,
        length: 0x2000,
    };

    pub const TITLE_TILEMAP: Record = Record {
        address: 0x08E000,
        length: 0x800,
    };

    pub const TITLE_PALETTE: Record = Record {
        address: 0x08E800,
        length: 0x100,
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
