use speedy::{Readable, Writable};

#[derive(Debug, PartialEq, Eq, Readable, Writable)]
#[speedy(tag_type = u32)]
pub enum TopicKind_t {
    NO_KEY = 1,
    WITH_KEY = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    serialization_test!(type = TopicKind_t,
    {
        topic_kind_t_no_key,
        TopicKind_t::NO_KEY,
        le = [0x01, 0x00, 0x00, 0x00],
        be = [0x00, 0x00, 0x00, 0x01]
    },
    {
        topic_kind_t_with_key,
        TopicKind_t::WITH_KEY,
        le = [0x02, 0x00, 0x00, 0x00],
        be = [0x00, 0x00, 0x00, 0x02]
    });
}
