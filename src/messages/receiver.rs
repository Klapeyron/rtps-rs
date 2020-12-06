use crate::common::validity_trait::Validity;
use crate::messages::heartbeat::Heartbeat;
use crate::messages::heartbeat_frag::HeartbeatFrag;
use crate::messages::info_destination::InfoDestination;
use crate::messages::nack_frag::NackFrag;
use crate::messages::protocol_version::ProtocolVersion_t;
use crate::messages::submessage::EntitySubmessage;
use crate::messages::submessage_header::SubmessageHeader;
use crate::messages::submessage_kind::SubmessageKind;
use crate::messages::vendor_id::VendorId_t;
use crate::messages::{ack_nack::AckNack, gap::Gap, header::Header, info_source::InfoSource};
use crate::structure::guid_prefix::GuidPrefix_t;
use crate::structure::locator::{LocatorKind_t, LocatorList_t, Locator_t};
use crate::structure::time::Time_t;

use log::info;
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

impl Receiver {
    pub fn new(locator_kind: LocatorKind_t) -> Self {
        Receiver {
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
        }
    }
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
            receiver: Receiver::new(locator_kind),
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
                    if submessage_header.submessage_length == 0
                        && submessage_header.submessage_id != SubmessageKind::INFO_TS
                        && submessage_header.submessage_id != SubmessageKind::PAD
                    {
                        // This is a last submessage
                        self.state = DeserializationState::ReadingHeader;
                    }
                    Ok(submessage_header)
                })
                .and_then(|submessage_header| match submessage_header.submessage_id {
                    SubmessageKind::ACKNACK => {
                        let ack_nack = AckNack::read_from_buffer_owned_with_ctx(
                            submessage_header.flags.endianness_flag(),
                            &bytes.split_to(submessage_header.submessage_length.into()),
                        )?;
                        Ok(Some(EntitySubmessage::AckNack(
                            ack_nack,
                            submessage_header.flags,
                        )))
                    }
                    SubmessageKind::DATA => {
                        unimplemented!();
                    }
                    SubmessageKind::DATA_FRAG => {
                        unimplemented!();
                    }
                    SubmessageKind::GAP => {
                        let gap = Gap::read_from_buffer_owned_with_ctx(
                            submessage_header.flags.endianness_flag(),
                            &bytes.split_to(submessage_header.submessage_length.into()),
                        )?;
                        Ok(Some(EntitySubmessage::Gap(gap)))
                    }
                    SubmessageKind::NACK_FRAG => {
                        let nack_frag = NackFrag::read_from_buffer_owned_with_ctx(
                            submessage_header.flags.endianness_flag(),
                            &bytes.split_to(submessage_header.submessage_length.into()),
                        )?;

                        Ok(Some(EntitySubmessage::NackFrag(nack_frag)))
                    }
                    SubmessageKind::HEARTBEAT => {
                        let heartbeat = Heartbeat::read_from_buffer_owned_with_ctx(
                            submessage_header.flags.endianness_flag(),
                            &bytes.split_to(submessage_header.submessage_length.into()),
                        )?;

                        Ok(Some(EntitySubmessage::Heartbeat(
                            heartbeat,
                            submessage_header.flags,
                        )))
                    }
                    SubmessageKind::HEARTBEAT_FRAG => {
                        let heartbeat_frag = HeartbeatFrag::read_from_buffer_owned_with_ctx(
                            submessage_header.flags.endianness_flag(),
                            &bytes.split_to(submessage_header.submessage_length.into()),
                        )?;

                        Ok(Some(EntitySubmessage::HeartbeatFrag(heartbeat_frag)))
                    }
                    SubmessageKind::INFO_SRC => {
                        let info_src = InfoSource::read_from_buffer_owned_with_ctx(
                            submessage_header.flags.endianness_flag(),
                            &bytes.split_to(submessage_header.submessage_length.into()),
                        )?;
                        self.receiver.source_guid_prefix = info_src.guid_prefix;
                        self.receiver.source_version = info_src.protocol_version;
                        self.receiver.source_vendor_id = info_src.vendor_id;
                        self.receiver.unicast_reply_locator_list = vec![Locator_t::LOCATOR_INVALID];
                        self.receiver.multicast_reply_locator_list =
                            vec![Locator_t::LOCATOR_INVALID];
                        self.receiver.have_timestamp = false;

                        Ok(None)
                    }
                    SubmessageKind::INFO_DST => {
                        let info_dst = InfoDestination::read_from_buffer_owned_with_ctx(
                            submessage_header.flags.endianness_flag(),
                            &bytes.split_to(submessage_header.submessage_length.into()),
                        )?;

                        if info_dst.guid_prefix != GuidPrefix_t::GUIDPREFIX_UNKNOWN {
                            self.receiver.dest_guid_prefix = info_dst.guid_prefix;
                        }

                        Ok(None)
                    }
                    SubmessageKind::INFO_REPLAY => {
                        let mut bytes = bytes.split_to(submessage_header.submessage_length.into());
                        let (unicast_locator_list, read_bytes) =
                            LocatorList_t::read_with_length_from_buffer_with_ctx(
                                submessage_header.flags.endianness_flag(),
                                &bytes,
                            );
                        self.receiver.unicast_reply_locator_list = unicast_locator_list?;

                        use crate::bytes::Buf;
                        let mut bytes = bytes.split_off(read_bytes);

                        self.receiver.multicast_reply_locator_list =
                            if submessage_header.flags.is_flag_set(0x02) {
                                let (multicast_locator_list, read_bytes) =
                                    LocatorList_t::read_with_length_from_buffer_with_ctx(
                                        submessage_header.flags.endianness_flag(),
                                        &bytes,
                                    );
                                bytes.advance(read_bytes);
                                multicast_locator_list?
                            } else {
                                vec![]
                            };

                        Ok(None)
                    }
                    SubmessageKind::INFO_TS => {
                        if !submessage_header.flags.is_flag_set(0x02) {
                            let timestamp = Time_t::read_from_buffer_owned_with_ctx(
                                submessage_header.flags.endianness_flag(),
                                &bytes.split_to(submessage_header.submessage_length.into()),
                            )?;
                            self.receiver.have_timestamp = true;
                            self.receiver.timestamp = timestamp;
                        } else {
                            self.receiver.have_timestamp = false;
                        }

                        Ok(None)
                    }
                    SubmessageKind::PAD => {
                        use crate::bytes::Buf;
                        bytes.advance(submessage_header.submessage_length.into());
                        Ok(None)
                    }
                    _ => {
                        info!(
                            "Received unknown submessage with id {:?}, skipping",
                            submessage_header.submessage_id
                        );
                        Ok(None)
                    }
                })
                .or_else(|err| Err(err.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use speedy::Writable;

    use super::*;
    use crate::messages::fragment_number::FragmentNumber_t;
    use crate::messages::fragment_number_set::FragmentNumberSet_t;
    use crate::messages::header::Header;
    use crate::messages::submessage_flag::SubmessageFlag;
    use crate::structure::count::Count_t;
    use crate::structure::entity_id::EntityId_t;
    use crate::structure::sequence_number::SequenceNumber_t;
    use crate::structure::sequence_number_set::SequenceNumberSet_t;

    struct EntitySubmessageIterator<'a> {
        message_receiver: &'a mut MessageReceiver,
        bytes: bytes::BytesMut,
    }

    impl<'a> Iterator for EntitySubmessageIterator<'a> {
        type Item = Result<Option<EntitySubmessage>, std::io::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            Some(self.message_receiver.decode(&mut self.bytes))
        }
    }

    macro_rules! encode_message {
        (header = $header:expr,
            [$(submessage_header = $submessage_header:expr, submessage_entities = [ $($entity:expr),* ],)+]) => {{
                let mut serialized_input: Vec<u8> = $header.write_to_vec_with_ctx(Endianness::NATIVE).unwrap();
                    $(
                        let mut submessage_header = $submessage_header;
                        let mut _submessage_content: Vec<u8> = vec![];
                        $(
                            let serialized_submessage =
                                $entity.write_to_vec_with_ctx(submessage_header.flags.endianness_flag()).unwrap();
                            _submessage_content.extend(serialized_submessage.into_iter());
                        )*
                        let provided_submessage_length = submessage_header.submessage_length;
                        let calculated_submessage_length = _submessage_content.len() as u16;
                        assert_eq!(
                            provided_submessage_length, calculated_submessage_length,
                            "Try to replace provided submessage_length {} with {}.",
                            provided_submessage_length, calculated_submessage_length
                        );
                        submessage_header.submessage_length = calculated_submessage_length;
                        let submessage_header = submessage_header.write_to_vec_with_ctx(submessage_header.flags.endianness_flag()).unwrap();
                        serialized_input.extend(submessage_header.into_iter());
                        serialized_input.extend(_submessage_content.into_iter());
                    )+
                bytes::BytesMut::from(&serialized_input[..])
            }};
    }

    macro_rules! message_decoding_test {
        (test_name = $name:ident, bytes = $bytes:expr,
        expected_notifications = [ $($expected_notification:expr),* ]
        $(, receiver_state = $receiver:expr)?
        ) => {
            mod $name {
                use super::*;

                #[test]
                fn test_submessage_decoding() {
                    let mut message_receiver = MessageReceiver::new(LocatorKind_t::LOCATOR_KIND_INVALID);
                    let messages_iterator = EntitySubmessageIterator {
                        message_receiver: &mut message_receiver,
                        bytes: $bytes
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
                    $(pretty_assertions::assert_eq!(message_receiver.receiver, $receiver);)?
                }
            }
        }
    }

    message_decoding_test!(
        test_name = single_ack_nack_with_non_empty_info_ts,
        bytes = encode_message!(
            header = Header::new(GuidPrefix_t::GUIDPREFIX_UNKNOWN),
            [
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::INFO_TS,
                    flags: SubmessageFlag { flags: 0b0000_0000 },
                    submessage_length: 8,
                },
                submessage_entities = [Time_t::TIME_INFINITE],
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
                ],
            ]
        ),
        expected_notifications = [Ok(EntitySubmessage::AckNack(
            AckNack {
                reader_id: EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_READER,
                writer_id: EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_WRITER,
                reader_sn_state: SequenceNumberSet_t::new(SequenceNumber_t::from(0)),
                count: Count_t::from(1)
            },
            SubmessageFlag { flags: 0b0000_0000 }
        ))],
        receiver_state = Receiver {
            have_timestamp: true,
            timestamp: Time_t::TIME_INFINITE,
            ..Receiver::new(LocatorKind_t::LOCATOR_KIND_INVALID)
        }
    );

    message_decoding_test!(
        test_name = single_gap_with_info_src,
        bytes = encode_message!(
            header = Header::new(GuidPrefix_t::GUIDPREFIX_UNKNOWN),
            [
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::INFO_SRC,
                    flags: SubmessageFlag { flags: 0b0000_0001 },
                    submessage_length: 16,
                },
                submessage_entities = [
                    ProtocolVersion_t::PROTOCOLVERSION,
                    VendorId_t::VENDOR_UNKNOWN,
                    GuidPrefix_t::GUIDPREFIX_UNKNOWN
                ],
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::GAP,
                    flags: SubmessageFlag { flags: 0b0000_0000 },
                    submessage_length: 28,
                },
                submessage_entities = [
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
                    SequenceNumber_t::from(42),
                    SequenceNumberSet_t::new(SequenceNumber_t::from(0b10110100))
                ],
            ]
        ),
        expected_notifications = [Ok(EntitySubmessage::Gap(Gap {
            reader_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
            writer_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            gap_start: SequenceNumber_t::from(42),
            gap_list: SequenceNumberSet_t::new(SequenceNumber_t::from(0b10110100))
        }))],
        receiver_state = Receiver {
            dest_guid_prefix: GuidPrefix_t::GUIDPREFIX_UNKNOWN,
            source_version: ProtocolVersion_t::PROTOCOLVERSION,
            source_vendor_id: VendorId_t::VENDOR_UNKNOWN,
            unicast_reply_locator_list: vec![Locator_t::LOCATOR_INVALID],
            multicast_reply_locator_list: vec![Locator_t::LOCATOR_INVALID],
            have_timestamp: false,
            ..Receiver::new(LocatorKind_t::LOCATOR_KIND_INVALID)
        }
    );

    message_decoding_test!(
        test_name = single_heartbeat_with_info_dst,
        bytes = encode_message!(
            header = Header::new(GuidPrefix_t::GUIDPREFIX_UNKNOWN),
            [
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::INFO_DST,
                    flags: SubmessageFlag { flags: 0b0000_0001 },
                    submessage_length: 12,
                },
                submessage_entities = [GuidPrefix_t::from([0x42; 12])],
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::HEARTBEAT,
                    flags: SubmessageFlag { flags: 0b0000_0001 },
                    submessage_length: 28,
                },
                submessage_entities = [
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
                    SequenceNumber_t::from(7),
                    SequenceNumber_t::from(11),
                    Count_t::from(99)
                ],
            ]
        ),
        expected_notifications = [Ok(EntitySubmessage::Heartbeat(
            Heartbeat {
                reader_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
                writer_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
                first_sn: SequenceNumber_t::from(7),
                last_sn: SequenceNumber_t::from(11),
                count: Count_t::from(99)
            },
            SubmessageFlag { flags: 0b0000_0001 }
        ))],
        receiver_state = Receiver {
            dest_guid_prefix: GuidPrefix_t::from([0x42; 12]),
            ..Receiver::new(LocatorKind_t::LOCATOR_KIND_INVALID)
        }
    );

    message_decoding_test!(
        test_name = single_heartbeat_frag_with_info_reply_and_multicast_locator_list,
        bytes = encode_message!(
            header = Header::new(GuidPrefix_t::GUIDPREFIX_UNKNOWN),
            [
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::INFO_REPLAY,
                    flags: SubmessageFlag { flags: 0b0000_0011 },
                    submessage_length: 80,
                },
                submessage_entities = [
                    vec![Locator_t::LOCATOR_INVALID],
                    vec![
                        Locator_t::from("127.0.0.1:8080".parse::<std::net::SocketAddr>().unwrap()),
                        Locator_t::from(
                            "[2001:db8::1]:8080"
                                .parse::<std::net::SocketAddr>()
                                .unwrap()
                        )
                    ]
                ],
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::HEARTBEAT_FRAG,
                    flags: SubmessageFlag { flags: 0b0000_0000 },
                    submessage_length: 24,
                },
                submessage_entities = [
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
                    SequenceNumber_t::from(36),
                    FragmentNumber_t::from(33),
                    Count_t::from(12345)
                ],
            ]
        ),
        expected_notifications = [Ok(EntitySubmessage::HeartbeatFrag(HeartbeatFrag {
            reader_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
            writer_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            writer_sn: SequenceNumber_t::from(36),
            last_fragment_num: FragmentNumber_t::from(33),
            count: Count_t::from(12345)
        }))],
        receiver_state = Receiver {
            unicast_reply_locator_list: vec![Locator_t::LOCATOR_INVALID],
            multicast_reply_locator_list: vec![
                Locator_t::from("127.0.0.1:8080".parse::<std::net::SocketAddr>().unwrap()),
                Locator_t::from(
                    "[2001:db8::1]:8080"
                        .parse::<std::net::SocketAddr>()
                        .unwrap()
                )
            ],
            ..Receiver::new(LocatorKind_t::LOCATOR_KIND_INVALID)
        }
    );

    message_decoding_test!(
        test_name = single_nack_frag_with_pad,
        bytes = encode_message!(
            header = Header::new(GuidPrefix_t::GUIDPREFIX_UNKNOWN),
            [
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::PAD,
                    flags: SubmessageFlag { flags: 0b0000_0000 },
                    submessage_length: 0,
                },
                submessage_entities = [],
                submessage_header = SubmessageHeader {
                    submessage_id: SubmessageKind::NACK_FRAG,
                    flags: SubmessageFlag { flags: 0b0000_0000 },
                    submessage_length: 28,
                },
                submessage_entities = [
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
                    EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
                    SequenceNumber_t::from(69),
                    FragmentNumberSet_t::new(FragmentNumber_t::from(96)),
                    Count_t::from(54321)
                ],
            ]
        ),
        expected_notifications = [Ok(EntitySubmessage::NackFrag(NackFrag {
            reader_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
            writer_id: EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            writer_sn: SequenceNumber_t::from(69),
            fragment_number_state: FragmentNumberSet_t::new(FragmentNumber_t::from(96)),
            count: Count_t::from(54321)
        }))],
        receiver_state = Receiver::new(LocatorKind_t::LOCATOR_KIND_INVALID)
    );

    message_decoding_test!(
        test_name = wireshark_ack_nack_with_info_src,
        bytes = BytesMut::from(
            &[
                0x52, 0x54, 0x50, 0x53, 0x02, 0x01, 0x01, 0x0f, 0x01, 0x0f, 0xbb, 0x1d, 0xdf, 0x2b,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x01, 0x0c, 0x00, 0x01, 0x0f, 0xbb, 0x1d,
                0xe6, 0x2b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x01, 0x18, 0x00, 0x00, 0x00,
                0x04, 0xc7, 0x00, 0x00, 0x04, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00
            ][..]
        ),
        expected_notifications = [Ok(EntitySubmessage::AckNack(
            AckNack {
                reader_id: EntityId_t::ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_READER,
                writer_id: EntityId_t::ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_WRITER,
                reader_sn_state: SequenceNumberSet_t::new(SequenceNumber_t::from(0)),
                count: Count_t::from(1)
            },
            SubmessageFlag { flags: 0b0000_0001 }
        ))],
        receiver_state = Receiver {
            source_guid_prefix: GuidPrefix_t::from([
                0x01, 0x0f, 0xbb, 0x1d, 0xdf, 0x2b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]),
            dest_guid_prefix: GuidPrefix_t::from([
                0x01, 0x0f, 0xbb, 0x1d, 0xe6, 0x2b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]),
            source_version: ProtocolVersion_t::PROTOCOLVERSION_2_1,
            source_vendor_id: VendorId_t::from([0x01, 0x0F]),
            ..Receiver::new(LocatorKind_t::LOCATOR_KIND_INVALID)
        }
    );
}
