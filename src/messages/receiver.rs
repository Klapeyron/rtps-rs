use crate::common::validity_trait::Validity;
use crate::messages::ack_nack::AckNack;
use crate::messages::header::Header;
use crate::messages::protocol_version::ProtocolVersion_t;
use crate::messages::submessage::EntitySubmessage;
use crate::messages::submessage_header::SubmessageHeader;
use crate::messages::submessage_kind::SubmessageKind;
use crate::messages::vendor_id::VendorId_t;
use crate::structure::guid_prefix::GuidPrefix_t;
use crate::structure::locator::{LocatorKind_t, LocatorList_t, Locator_t};

use crate::structure::time::Time_t;
use speedy::{Endianness, Readable};
use std::io::{Error, ErrorKind};

use bytes::BytesMut;
use tokio_util::codec::Decoder;

#[derive(Debug, PartialEq)]
pub struct Receiver {
    pub source_version: ProtocolVersion_t,
    pub source_vendor_id: VendorId_t,
    pub source_guid_prefix: GuidPrefix_t,
    pub dest_guid_prefix: GuidPrefix_t,
    pub unicast_reply_locator_list: LocatorList_t,
    pub multicast_reply_locator_list: LocatorList_t,
    pub have_timestamp: bool,
    pub timestamp: Time_t,
}

enum DeserializationState {
    ReadingHeader,
    ReadingSubmessage,
}

pub struct MessageReceiver {
    receiver: Receiver,
    state: DeserializationState,
}

impl MessageReceiver {
    pub fn new(locator_kind: LocatorKind_t) -> Self {
        MessageReceiver {
            receiver: Receiver {
                source_version: ProtocolVersion_t::PROTOCOLVERSION,
                source_vendor_id: VendorId_t::VENDOR_UNKNOWN,
                source_guid_prefix: GuidPrefix_t::GUIDPREFIX_UNKNOWN,
                dest_guid_prefix: GuidPrefix_t::GUIDPREFIX_UNKNOWN,
                unicast_reply_locator_list: vec![Locator_t {
                    kind: locator_kind,
                    address: Locator_t::LOCATOR_ADDRESS_INVALID,
                    port: Locator_t::LOCATOR_PORT_INVALID,
                }],
                multicast_reply_locator_list: vec![Locator_t {
                    kind: locator_kind,
                    address: Locator_t::LOCATOR_ADDRESS_INVALID,
                    port: Locator_t::LOCATOR_PORT_INVALID,
                }],
                have_timestamp: false,
                timestamp: Time_t::TIME_INVALID,
            },
            state: DeserializationState::ReadingHeader,
        }
    }
}

impl Decoder for MessageReceiver {
    type Item = EntitySubmessage;
    type Error = std::io::Error;

    fn decode(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let validate_header = |header: Header| {
            if header.valid() {
                Ok(header)
            } else {
                Err(speedy::Error::custom("Invalid data".to_owned()))
            }
        };

        match self.state {
            DeserializationState::ReadingHeader => Header::read_from_buffer_owned_with_ctx(
                Endianness::NATIVE,
                &bytes.split_to(<Header as Readable<Endianness>>::minimum_bytes_needed()),
            )
            .and_then(validate_header)
            .and_then(|header: Header| {
                self.receiver.source_guid_prefix = header.guid_prefix;
                self.receiver.source_version = header.protocol_version;
                self.receiver.source_vendor_id = header.vendor_id;
                self.receiver.have_timestamp = false;

                self.state = DeserializationState::ReadingSubmessage;
                Ok(None)
            })
            .or_else(|err| {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Header parsing error: {:?}", err),
                ))
            }),

            DeserializationState::ReadingSubmessage => {
                SubmessageHeader::read_from_buffer_owned_with_ctx(
                    Endianness::NATIVE,
                    &bytes.split_to(
                        <SubmessageHeader as Readable<Endianness>>::minimum_bytes_needed(),
                    ),
                )
                .and_then(|submessage_header| {
                    match submessage_header.submessage_id {
                        SubmessageKind::ACKNACK => {
                            let ack_nack = AckNack::read_from_buffer_owned_with_ctx(
                                submessage_header.flags.endianness_flag(),
                                &bytes,
                            )?;
                            Ok(Some(EntitySubmessage::AckNack(
                                ack_nack,
                                submessage_header.flags,
                            )))
                        }
                        /*
                        SubmessageKind::DATA => Ok(None),
                        SubmessageKind::DATA_FRAG => Ok(None),
                        SubmessageKind::GAP => Ok(None),
                        SubmessageKind::HEARTBEAT => Ok(None),
                        SubmessageKind::HEARTBEAT_FRAG => Ok(None),
                        SubmessageKind::NACK_FRAG => Ok(None),
                        SubmessageKind::INFO_DST => Ok(None),
                        */
                        // TODO: skip this submessage and go to another one
                        _ => unimplemented!(),
                    }
                })
                .or_else(|err| Err(err.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::header::Header;
    use crate::messages::submessage_flag::SubmessageFlag;
    use crate::structure::count::Count_t;
    use crate::structure::entity_id::EntityId_t;
    use crate::structure::sequence_number::SequenceNumber_t;
    use crate::structure::sequence_number_set::SequenceNumberSet_t;

    struct EntitySubmessageIterator {
        message_receiver: MessageReceiver,
        bytes: bytes::BytesMut,
    }

    impl Iterator for EntitySubmessageIterator {
        type Item = Result<Option<EntitySubmessage>, std::io::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            Some(self.message_receiver.decode(&mut self.bytes))
        }
    }

    macro_rules! message_decoding_test {
        (test_name = $name:ident, header = $header:expr,
        [$(submessage_header = $submessage_header:expr, submessage_entities = [ $($entity:expr),* ]),+],
        expected_notifications = [ $($expected_notification:expr),* ]) => {
            mod $name {
                use super::*;
                use speedy::{Writable};

                fn serialize_into_bytes() -> bytes::BytesMut {
                    let mut serialized_input: Vec<u8> = $header.write_to_vec_with_ctx(Endianness::NATIVE).unwrap();
                    $(
                        let mut submessage_header = $submessage_header;
                        let mut submessage_content: Vec<u8> = vec![];
                        $(
                            let serialized_submessage =
                                $entity.write_to_vec_with_ctx(submessage_header.flags.endianness_flag()).unwrap();
                            submessage_content.extend(serialized_submessage.into_iter());
                        )*
                        submessage_header.submessage_length = submessage_content.len() as u16;
                        let submessage_header = submessage_header.write_to_vec_with_ctx(submessage_header.flags.endianness_flag()).unwrap();
                        serialized_input.extend(submessage_header.into_iter());
                        serialized_input.extend(submessage_content.into_iter());
                    )+
                    bytes::BytesMut::from(&serialized_input[..])
                }

                #[test]
                fn test_submessage_decoding() {
                    let messages_iterator = EntitySubmessageIterator {
                        message_receiver: MessageReceiver::new(LocatorKind_t::LOCATOR_KIND_INVALID),
                        bytes: serialize_into_bytes()
                    };

                    let expected_notifications = vec![$($expected_notification),*]
                        .into_iter()
                        .inspect(|expectation| {
                            println!("Expected notification: {:#?}", expectation)
                        });

                    let decoder_output = messages_iterator
                        // Decoder impl is not greedy right now, which means it returns Ok(None)
                        // after it parses a header
                        .take(10*expected_notifications.len())
                        .filter(|maybe_message| match maybe_message {
                            Ok(None) => false,
                            _ => true
                        })
                        .map(|maybe_parsed_message|
                            match maybe_parsed_message {
                                Ok(Some(parsed_message)) => Ok(parsed_message),
                                Err(error) => Err((error.kind(), format!("Description: {:?}", error))),
                                Ok(None) => unreachable!()
                            }
                        )
                        .inspect(|parsed_message| {
                            println!("Parsed message: {:#?}", parsed_message)
                        })
                        // Let's take only amount of parsed messages/errors that matches to expectations
                        // we do not care what happends after that :)
                        .take(expected_notifications.len());
                    assert!(decoder_output.eq(expected_notifications));
                }
            }
        }
    }

    message_decoding_test!(
        test_name = single_ack_nack,
        header = Header::new(GuidPrefix_t::GUIDPREFIX_UNKNOWN),
        [
            submessage_header = SubmessageHeader {
                submessage_id: SubmessageKind::ACKNACK,
                flags: SubmessageFlag { flags: 0b0000_0000 },
                submessage_length: 24,
            },
            submessage_entities = [
                EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_READER,
                EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_WRITER,
                SequenceNumberSet_t::new(SequenceNumber_t::from(0)),
                Count_t::from(1)
            ]
        ],
        expected_notifications = [Ok(EntitySubmessage::AckNack(
            AckNack {
                reader_id: EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_READER,
                writer_id: EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_WRITER,
                reader_sn_state: SequenceNumberSet_t::new(SequenceNumber_t::from(0)),
                count: Count_t::from(1)
            },
            SubmessageFlag { flags: 0b0000_0000 }
        ))]
    );
}
