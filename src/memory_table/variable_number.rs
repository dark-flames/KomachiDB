pub fn encode_usize(mut num: usize) -> Box<[u8]> {
    let mut slice = vec![];
    while num >= (1 << 7) {
        slice.push((num | (1 << 7)) as u8);
        num >>= 7;
    }

    slice.push((num & (!(1 << 7))) as u8);

    Vec::into_boxed_slice(slice)
}

pub fn decode_usize(ptr: *const u8) -> (usize, *const u8) {
    let mut current_ptr = ptr;
    let mut slice = vec![];

    loop {
        let byte = unsafe { *current_ptr };
        current_ptr = unsafe { current_ptr.add(1) };

        if (byte & (1 << 7)) == 0 {
            slice.push(byte);
            break;
        } else {
            slice.push(byte & (!(1 << 7)));
        }
    }

    slice.reverse();

    (
        slice
            .into_iter()
            .fold(0, |carry, slice| (carry << 7) | slice as usize),
        current_ptr,
    )
}

#[test]
fn test_encode() {
    use rand::random;
    let num = random();
    let a = encode_usize(num);
    let result = decode_usize(a.as_ref().as_ptr());

    assert_eq!(result.0, num);
}
