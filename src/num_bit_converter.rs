pub fn num_to_bit<const B: usize>(num: usize) -> [bool; B] {
    let mut result = [false; B];
    result.iter_mut()
        .enumerate()
        .for_each(|(i, v)| *v = num & (1 << i) != 0);
    result
}

pub fn bit_to_num<const B: usize>(bits: [bool; B]) -> usize {
    bits.iter()
        .enumerate()
        .map(|(i, &b)| (1 << i) * if b {1} else {0})
        .sum()
}

#[test]
fn conv_conv_eq_id_until256() {
    for i in 0..256 {
        assert_eq!(i, bit_to_num(num_to_bit::<8>(i)));
    }
}
