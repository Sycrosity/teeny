#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum SpotifyMiniError {
    I2C,
    Unknown,
    IntegerOverflow,
    InterfaceError,
}
