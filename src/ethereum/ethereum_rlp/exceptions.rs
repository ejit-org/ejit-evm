
#[derive(Debug, PartialEq)]
/// Common base class for all RLP exceptions.
pub enum RLPException {
    /// Indicates that RLP decoding failed.
    DecodingError(&'static str),
    /// Indicates that RLP encoding failed.
    EncodingError(&'static str),

    /// Buffer not big enough
    DestTooSmall(usize),
}
