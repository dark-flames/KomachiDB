use crate::interface::Key;

pub struct Block<'pool, K: Key> {
    pub block_ref: &'pool [u8],
    pub key_size: &'pool u32,
    pub key: &'pool K,
    pub value_size: &'pool u32,
    pub value: &'pool [u8],
}
