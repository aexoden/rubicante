
pub const GAME_TITLE_ADDRESS: usize = 0x00FFC0;
pub const GAME_TITLE_LENGTH: usize = 21;

const HASH_USA: &str = "680535dc1c4196c53b40dc9c2c9bc159a77802ab8d4b474bef5dc0281c15ad06";
const HASH_USA_REV_A: &str = "414bacc05a18a6137c0de060b4094ab6d1b75105342b0bb36a42e45d945a0e4d";

#[derive(Copy, Clone)]
pub enum Version {
    Us,
    UsRevA
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
