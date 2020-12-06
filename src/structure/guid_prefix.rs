use speedy::{Context, Readable, Reader, Writable, Writer};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct GuidPrefix_t {
    pub entity_key: [u8; 12],
}

impl GuidPrefix_t {
    pub const GUIDPREFIX_UNKNOWN: GuidPrefix_t = GuidPrefix_t {
        entity_key: [0x00; 12],
    };
}

impl Default for GuidPrefix_t {
    fn default() -> GuidPrefix_t {
        GuidPrefix_t::GUIDPREFIX_UNKNOWN
    }
}

impl From<[u8; 12]> for GuidPrefix_t {
    fn from(entity_key: [u8; 12]) -> Self {
        GuidPrefix_t {
            entity_key: entity_key,
        }
    }
}

impl<'a, C: Context> Readable<'a, C> for GuidPrefix_t {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let mut guid_prefix = GuidPrefix_t::default();
        for i in 0..guid_prefix.entity_key.len() {
            guid_prefix.entity_key[i] = reader.read_u8()?;
        }
        Ok(guid_prefix)
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl<C: Context> Writable<C> for GuidPrefix_t {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        for elem in &self.entity_key {
            writer.write_u8(*elem)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speedy::Endianness;

    #[test]
    fn minimum_bytes_needed() {
        assert_eq!(
            12,
            <GuidPrefix_t as Readable<Endianness>>::minimum_bytes_needed()
        );
    }

    serialization_test!( type = GuidPrefix_t,
    {
        guid_prefix_unknown,
        GuidPrefix_t::GUIDPREFIX_UNKNOWN,
        le = [0x00; 12],
        be = [0x00; 12]
    },
    {
        guid_prefix_default,
        GuidPrefix_t::default(),
        le = [0x00; 12],
        be = [0x00; 12]
    },
    {
        guid_prefix_endianness_insensitive,
        GuidPrefix_t {
            entity_key: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
                        0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]
        },
        le = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
              0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB],
        be = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
              0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB]
    });
}
