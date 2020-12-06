use speedy::{Context, Readable, Reader, Writable, Writer};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub struct EntityId_t {
    entity_key: [u8; 3],
    entity_kind: u8,
}

impl EntityId_t {
    pub const ENTITYID_UNKNOWN: EntityId_t = EntityId_t {
        entity_key: [0x00; 3],
        entity_kind: 0x00,
    };
    pub const ENTITYID_PARTICIPANT: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x00, 0x01],
        entity_kind: 0xC1,
    };
    pub const ENTITYID_SEDP_BUILTIN_TOPIC_WRITER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x00, 0x02],
        entity_kind: 0xC2,
    };
    pub const ENTITYID_SEDP_BUILTIN_TOPIC_READER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x00, 0x02],
        entity_kind: 0xC7,
    };
    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_WRITER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x00, 0x03],
        entity_kind: 0xC2,
    };
    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_READER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x00, 0x03],
        entity_kind: 0xC7,
    };
    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_WRITER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x00, 0x04],
        entity_kind: 0xC2,
    };
    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_READER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x00, 0x04],
        entity_kind: 0xC7,
    };
    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x01, 0x00],
        entity_kind: 0xC2,
    };
    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x01, 0x00],
        entity_kind: 0xC7,
    };
    pub const ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x02, 0x00],
        entity_kind: 0xC2,
    };
    pub const ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId_t = EntityId_t {
        entity_key: [0x00, 0x02, 0x00],
        entity_kind: 0xC7,
    };
}

impl Default for EntityId_t {
    fn default() -> EntityId_t {
        EntityId_t::ENTITYID_UNKNOWN
    }
}

impl<'a, C: Context> Readable<'a, C> for EntityId_t {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let entity_key = [reader.read_u8()?, reader.read_u8()?, reader.read_u8()?];
        let entity_kind = reader.read_u8()?;
        Ok(EntityId_t {
            entity_key,
            entity_kind,
        })
    }
}

impl<C: Context> Writable<C> for EntityId_t {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        for elem in &self.entity_key {
            writer.write_u8(*elem)?
        }
        writer.write_u8(self.entity_kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    serialization_test!( type = EntityId_t,
        {
            entity_unknown,
            EntityId_t::ENTITYID_UNKNOWN,
            le = [0x00, 0x00, 0x00, 0x00],
            be = [0x00, 0x00, 0x00, 0x00]
        },
        {
            entity_default,
            EntityId_t::default(),
            le = [0x00, 0x00, 0x00, 0x00],
            be = [0x00, 0x00, 0x00, 0x00]
        },
        {
            entity_participant,
            EntityId_t::ENTITYID_PARTICIPANT,
            le = [0x00, 0x00, 0x01, 0xC1],
            be = [0x00, 0x00, 0x01, 0xC1]
        },
        {
            entity_sedp_builtin_topic_writer,
            EntityId_t::ENTITYID_SEDP_BUILTIN_TOPIC_WRITER,
            le = [0x00, 0x00, 0x02, 0xC2],
            be = [0x00, 0x00, 0x02, 0xC2]
        },
        {
            entity_sedp_builtin_topic_reader,
            EntityId_t::ENTITYID_SEDP_BUILTIN_TOPIC_READER,
            le = [0x00, 0x00, 0x02, 0xC7],
            be = [0x00, 0x00, 0x02, 0xC7]
        },
        {
            entity_sedp_builtin_publications_writer,
            EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_WRITER,
            le = [0x00, 0x00, 0x03, 0xC2],
            be = [0x00, 0x00, 0x03, 0xC2]
        },
        {
            entity_sedp_builtin_publications_reader,
            EntityId_t::ENTITYID_SEDP_BUILTIN_PUBLICATIONS_READER,
            le = [0x00, 0x00, 0x03, 0xC7],
            be = [0x00, 0x00, 0x03, 0xC7]
        },
        {
            entity_sedp_builtin_subscriptions_writer,
            EntityId_t::ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_WRITER,
            le = [0x00, 0x00, 0x04, 0xC2],
            be = [0x00, 0x00, 0x04, 0xC2]
        },
        {
            entity_sedp_builtin_subscriptions_reader,
            EntityId_t::ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_READER,
            le = [0x00, 0x00, 0x04, 0xC7],
            be = [0x00, 0x00, 0x04, 0xC7]
        },
        {
            entity_spdp_builtin_participant_writer,
            EntityId_t::ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
            le = [0x00, 0x01, 0x00, 0xC2],
            be = [0x00, 0x01, 0x00, 0xC2]
        },
        {
            entity_spdp_builtin_participant_reader,
            EntityId_t::ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            le = [0x00, 0x01, 0x00, 0xC7],
            be = [0x00, 0x01, 0x00, 0xC7]
        },
        {
            entity_p2p_builtin_participant_message_writer,
            EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            le = [0x00, 0x02, 0x00, 0xC2],
            be = [0x00, 0x02, 0x00, 0xC2]
        },
        {
            entity_p2p_builtin_participant_message_reader,
            EntityId_t::ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
            le = [0x00, 0x02, 0x00, 0xC7],
            be = [0x00, 0x02, 0x00, 0xC7]
        }
    );
}
