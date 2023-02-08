use super::{builder::Builder, Slab};
use bincode::de::{BorrowDecoder, Decoder};
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{BorrowDecode, Decode, Encode};

impl<T: Encode> Encode for Slab<T> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (self.len() as u64).encode(encoder)?;
        for (key, value) in self {
            (key as u64).encode(encoder)?;
            value.encode(encoder)?;
        }
        Ok(())
    }
}

impl<T: Decode> Decode for Slab<T> {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = u64::decode(decoder)? as usize;
        decoder.claim_container_read::<(u64, T)>(len)?;

        let mut builder = Builder::with_capacity(len);
        for _ in 0..len {
            decoder.unclaim_bytes_read(core::mem::size_of::<(u64, T)>());

            let key = u64::decode(decoder)? as usize;
            let value = T::decode(decoder)?;
            builder.pair(key, value);
        }
        Ok(builder.build())
    }
}

impl<'de, T: BorrowDecode<'de>> BorrowDecode<'de> for Slab<T> {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let len = u64::decode(decoder)? as usize;
        decoder.claim_container_read::<(u64, T)>(len)?;

        let mut builder = Builder::with_capacity(len);
        for _ in 0..len {
            decoder.unclaim_bytes_read(core::mem::size_of::<(u64, T)>());

            let key = u64::borrow_decode(decoder)? as usize;
            let value = T::borrow_decode(decoder)?;
            builder.pair(key, value);
        }
        Ok(builder.build())
    }
}
