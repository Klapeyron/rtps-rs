macro_rules! serialization_test {
    (type = $type:ty, $({ $name:ident , $original:expr, le = $le:expr, be = $be:expr }),+) => {
        $(mod $name {
            extern crate speedy;

            use super::*;
            use self::speedy::{Readable, Writable, Endianness};

            #[test]
            fn serialize_little_endian()
            {
                let original: $type = $original;
                let serialized = original.write_to_vec(Endianness::LittleEndian).unwrap();
                assert_eq!(serialized, $le);
            }

            #[test]
            fn serialize_big_endian()
            {
                let original: $type = $original;
                let serialized = original.write_to_vec(Endianness::BigEndian).unwrap();
                assert_eq!(serialized, $be);
            }

            #[test]
            fn serialize_deserialize_little_endian()
            {
                let original: $type = $original;

                let serialized = original.write_to_vec(Endianness::LittleEndian).unwrap();
                let deserialized: $type = Readable::read_from_buffer(Endianness::LittleEndian, &serialized).unwrap();

                assert_eq!(original, deserialized);
            }

            #[test]
            fn serialize_deserialize_big_endian() {
                let original: $type = $original;

                let serialized = original.write_to_vec(Endianness::BigEndian).unwrap();
                let deserialized: $type = Readable::read_from_buffer(Endianness::BigEndian, &serialized).unwrap();

                assert_eq!(original, deserialized);
            }
        })+
    }
}
