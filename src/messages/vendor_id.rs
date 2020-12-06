use speedy::{Context, Readable, Reader, Writable, Writer};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VendorId_t {
    pub vendor_id: [u8; 2],
}

impl VendorId_t {
    pub const VENDOR_UNKNOWN: VendorId_t = VendorId_t {
        vendor_id: [0x00; 2],
    };
}

impl From<[u8; 2]> for VendorId_t {
    fn from(vendor_id: [u8; 2]) -> Self {
        VendorId_t {
            vendor_id: vendor_id,
        }
    }
}

impl Default for VendorId_t {
    fn default() -> Self {
        VendorId_t::VENDOR_UNKNOWN
    }
}

impl<'a, C: Context> Readable<'a, C> for VendorId_t {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let mut vendor_id = VendorId_t::default();
        for i in 0..vendor_id.vendor_id.len() {
            vendor_id.vendor_id[i] = reader.read_u8()?;
        }
        Ok(vendor_id)
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        std::mem::size_of::<Self>()
    }
}

impl<C: Context> Writable<C> for VendorId_t {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        for elem in &self.vendor_id {
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
            2,
            <VendorId_t as Readable<Endianness>>::minimum_bytes_needed()
        );
    }

    serialization_test!( type = VendorId_t,
    {
        vendor_unknown,
        VendorId_t::VENDOR_UNKNOWN,
        le = [0x00, 0x00],
        be = [0x00, 0x00]
    });
}
