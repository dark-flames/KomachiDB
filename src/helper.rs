#[cfg(test)]
pub mod test {
    use bytes::Bytes;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use std::mem::size_of;
    use std::ptr::slice_from_raw_parts;

    pub fn generate_data(start: u32, end: u32) -> Vec<(u32, Bytes)> {
        let mut data: Vec<u32> = (start as u32..end as u32).collect();
        let mut rng = thread_rng();
        data.shuffle(&mut rng);
        data.into_iter().map(|k| (k, get_bytes(k))).collect()
    }

    pub fn get_u32(bytes: &[u8]) -> u32 {
        unsafe { *(bytes.as_ptr() as *const u32) }
    }

    pub fn get_bytes(n: u32) -> Bytes {
        let ptr = Box::into_raw(Box::new(n)) as *const u8;
        Bytes::copy_from_slice(unsafe {
            slice_from_raw_parts(ptr, size_of::<u32>())
                .as_ref()
                .unwrap()
        })
    }
}
