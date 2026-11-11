//! A reader default uses system endianness
//! If you will use another endianness,use set_endian.
//! 0.0.10 StreamReader enable for wasm32

mod binary;
mod bytes;
mod stream;
pub use self::binary::BinaryReader;
pub use self::bytes::BytesReader;
pub use self::stream::StreamReader;
