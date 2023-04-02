use std::num;

use crate::core::*;
use crate::basic_comp::*;
use crate::num_bit_converter;

pub struct HalfAdder {
    adder: MergeLayers<2, 4, 2>,
}
impl Component<2, 2> for HalfAdder {
    fn eval(&self, input: [bool; 2]) -> [bool; 2] {
        self.adder.eval(input)
    }
}

impl HalfAdder {
    pub fn new() -> Self {
        let layer1 = Wiring::create([0, 1, 0, 1]);
        let layer2 = ConcatBlocks::create(
            [Box::new(XOR::<2>::new()),
            Box::new(And::<2>::new())]
        );
        let adder = MergeLayers::create(Box::new(layer1), Box::new(layer2));
        Self {adder}
    }
}

#[test]
fn half_adder_test() {
    let adder = HalfAdder::new();
    assert_eq!(adder.eval([false, false]), [false, false]);
    assert_eq!(adder.eval([true, false]), [true, false]);
    assert_eq!(adder.eval([false, true]), [true, false]);
    assert_eq!(adder.eval([true, true]), [false, true]);
}

pub struct FullAdder {
    adder: MergeLayers<3, 3, 2>,
}

impl Component<3, 2> for FullAdder {
    fn eval(&self, input: [bool; 3]) -> [bool; 2] {
        self.adder.eval(input)
    }
}

impl FullAdder {
    pub fn new() -> Self {
        let layer1 = ConcatDifferentShapeBlocks::create(
            Box::new(Buffer::new()),
            Box::new(HalfAdder::new()),
        );
        let layer2 = ConcatDifferentShapeBlocks::create(
            Box::new(HalfAdder::new()),
            Box::new(Buffer::new())
        );
        let layer3 = ConcatDifferentShapeBlocks::create(
            Box::new(Buffer::new()),
            Box::new(Or::<2>::new()),
        );
        let adder = MergeLayers::create(Box::new(layer1), Box::new(layer2))
            .connect_to(Box::new(layer3));
        Self {adder}
    }
}

#[test]
fn full_adder_test() {
    use crate::num_bit_converter::*;
    let adder = FullAdder::new();
    for i in 0..8 {
        let input = num_to_bit(i);
        let ans = num_to_bit::<2>(input.iter().map(|&b| b as usize).sum());
        assert_eq!(adder.eval(input), ans);
    }
}


pub struct EightBitFullAdder {
    adder: MergeLayers<17, 17, 9>,
}
impl Component<17, 9> for EightBitFullAdder {
    fn eval(&self, input: [bool; 17]) -> [bool; 9] {
        self.adder.eval(input)
    }
}
impl EightBitFullAdder {
    pub fn new() -> Self {
        let layer1 = Wiring::<16, 16>::zip::<8>();
        let layer1 = ConcatDifferentShapeBlocks::create(
            Box::new(Buffer::new()), Box::new(layer1)
        );
        let eight_bit_adder = RecurrentBlock::<1, 2, 1, 8>::create_from_fn(FullAdder::new);
        let adder = MergeLayers::create(
            Box::new(layer1),
            Box::new(eight_bit_adder)
        );
        Self {adder}
    }
}

#[test]
fn eight_bit_adder_test() {
    use crate::num_bit_converter::*;
    let adder = EightBitFullAdder::new();
    for i in 0..256 {
        for j in 0..256 {
            let input1 = num_to_bit(((i << 8) + j) << 1);
            let input2 = num_to_bit((((i << 8) + j) << 1) + 1);
            println!("{:?}", input2);
            assert_eq!(
                bit_to_num(adder.eval(input1)),
                i + j
            );
            assert_eq!(
                bit_to_num(adder.eval(input2)),
                i + j + 1
            );
        }
    }
}

pub struct NByteAdder<const N: usize>
where
    [(); 16 * N]: Sized,
    [(); 8 * N + 1]: Sized,
{
    adder: MergeLayers<{16 * N}, {8 * N + 1}, {8 * N}>,
}

impl<const N: usize> Component<{16 * N}, {8 * N}> for NByteAdder<N>
where
    [(); 16 * N]: Sized,
    [(); 8 * N + 1]: Sized,
{
    fn eval(&self, input: [bool; 16 * N]) -> [bool; 8 * N] {
        self.adder.eval(input)
    }
}

impl<const N: usize> NByteAdder<N>
where
    [(); 16 * N]: Sized,
    [(); 1 + 16 * N]: Sized,
    [(); 0 + 16 * N]: Sized,
    [(); 8 * N + 1]: Sized,
{
    fn new() -> Self {
        let in_wrapper = Wiring::<{16 * N}, {0 + 16 * N}>::wrapper();
        let layer1 = Wiring::<{16 * N}, {16 * N}>::zip_with_chunk::<8>();
        let layer1 = ConcatDifferentShapeBlocks::create(
            Box::new(Constant::<0, 1, false>::new()),
            Box::new(layer1)
        );
        let adder = RecurrentBlock::<1, 16, 8, N>::create_from_fn(
            EightBitFullAdder::new
        );
        let mut ignore_c_table = [0; 8 * N];
        ignore_c_table.iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = i);
        let ignore_c = Wiring::<{8 * N + 1}, {8 * N}>::create(ignore_c_table);

        let adder = MergeLayers::create(Box::new(in_wrapper), Box::new(layer1))
            .connect_to(Box::new(adder))
            .connect_to(Box::new(ignore_c));
        Self{ adder}
    }
}

#[test]
fn nbyte_adder_test() {
    use crate::num_bit_converter::*;
    let adder = NByteAdder::<2>::new();
    for i in 0..256 {
        for j in 0..256 {
            let i = (i << 8) + i;
            let j = (j << 8) + j;
            assert_eq!(
                bit_to_num(adder.eval(num_to_bit((i << 16) + j))),
                (i + j) % (1 << 16)
            );
        }
    }
}

pub struct EightBitSubtractor {
    subtractor: MergeLayers<16, 9, 8>,
}

impl Component<16, 8> for EightBitSubtractor {
    fn eval(&self, input: [bool; 16]) -> [bool; 8] {
        self.subtractor.eval(input)
    }
}

impl EightBitSubtractor {
    fn new() -> Self {
        let layer1 = {
            let buffer = ConcatBlocks::create(
                [Buffer::new(); 8].map(|c| Box::new(c) as Box<dyn Component<1, 1>>)
            );
            let not = ConcatBlocks::create(
                [Not::new(); 8].map(|c| Box::new(c) as Box<dyn Component<1, 1>>)
            );
            let rev = ConcatBlocks::create(
                [Box::new(buffer),
                Box::new(not)]
            );
            let plus_one = Constant::<0, 1, true>::new();
            ConcatDifferentShapeBlocks::create(Box::new(plus_one), Box::new(rev))
        };
        let layer2 = EightBitFullAdder::new();
        let mut layer3_table = [0; 8];
        layer3_table.iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = i);
        // 繰り上がりの桁を無視
        let layer3 = Wiring::create(layer3_table);

        let subtractor = MergeLayers::create(Box::new(layer1), Box::new(layer2))
            .connect_to(Box::new(layer3));
        Self {subtractor}
    }
}

#[test]
fn subtractor_test() {
    use crate::num_bit_converter::*;
    let sub = EightBitSubtractor::new();
    for i in 0..256 {
        for j in 0..(i + 1) {
            // 引く数があと
            assert_eq!(
                sub.eval(num_to_bit((j << 8) + i)),
                num_to_bit(i - j)
            );
        }
    }
}

struct Comparator {
    comp: MergeLayers<2, 4, 3>,
}

impl Component<2, 3> for Comparator {
    fn eval(&self, input: [bool; 2]) -> [bool; 3] {
        self.comp.eval(input)
    }
}

impl Comparator {
    fn new() -> Self {
        let layer1 = ConcatBlocks::create(
            [Branch::<2>::new(); 2].map(|b| Box::new(b) as Box<dyn Component<1, 2>>)
        );
        let layer2 = ConcatBlocks::create(
            [Box::new(Buffer::new()),
            Box::new(Not::new()),
            Box::new(Not::new()),
            Box::new(Buffer::new())]
        );
        let layer3 = Wiring::create([0, 2, 1, 3]);
        let layer4 = ConcatBlocks::create(
            [And::new(); 2].map(|c| Box::new(c) as Box<dyn Component<2, 1>>)
        );
        let layer5 = ConcatBlocks::create(
            [Branch::<2>::new(); 2].map(|b| Box::new(b) as Box<dyn Component<1, 2>>)
        );
        let nor = MergeLayers::create(
            Box::new(Or::<2>::new()),
            Box::new(Not::new())
        );
        let layer6 = {
            let x_eq = ConcatDifferentShapeBlocks::create(
                Box::new(Buffer::new()),
                Box::new(nor)
            );
            ConcatDifferentShapeBlocks::create(
                Box::new(x_eq),
                Box::new(Buffer::new())
            )
        };

        let comp = MergeLayers::create(Box::new(layer1), Box::new(layer2))
            .connect_to(Box::new(layer3))
            .connect_to(Box::new(layer4))
            .connect_to(Box::new(layer5))
            .connect_to(Box::new(layer6));

        Self { comp}
    }
}

#[test]
fn comparator_test() {
    let comp = Comparator::new();
    assert_eq!(comp.eval([false, false]), [false, true, false]);
    assert_eq!(comp.eval([true, false]), [true, false, false]);
    assert_eq!(comp.eval([false, true]), [false, false, true]);
    assert_eq!(comp.eval([true, true]), [false, true, false]);
}

struct EightBitComparator {
    comp: MergeLayers<19, 19, 3>,
}

impl Component<19, 3> for EightBitComparator {
    fn eval(&self, input: [bool; 19]) -> [bool; 3] {
        self.comp.eval(input)
    }
}

impl EightBitComparator {
    fn new() -> Self {
        let make_comp = || {
            let if_eq = {
                let layer1 = Wiring::<4, 6>::create([0, 1, 0, 2, 0, 3]);
                let layer2 = ConcatBlocks::create(
                    [And::<2>::new(); 3].map(|c| Box::new(c) as Box<dyn Component<2, 1>>)
                );
                MergeLayers::create(Box::new(layer1), Box::new(layer2))
            };

            let cur_bit_comp = Comparator::new();
            let recur_bit = ConcatBlocks::create(
                [Buffer::new(); 3].map(|c| Box::new(c) as Box<dyn Component<1, 1>>)
            );

            let layer1 = ConcatDifferentShapeBlocks::create(
                Box::new(recur_bit),
                Box::new(cur_bit_comp)
            );
            let layer2 = Wiring::<6, 6>::create([3, 4, 0, 1, 2, 5]);
            let layer3 = ConcatDifferentShapeBlocks::create(
                Box::new(Buffer::new()),
                Box::new(if_eq),
            );
            let layer3 = ConcatDifferentShapeBlocks::create(
                Box::new(layer3),
                Box::new(Buffer::new()),
            );
            let layer4 = ConcatDifferentShapeBlocks::create(
                Box::new(Or::<2>::new()),
                Box::new(Buffer::new()),
            );
            let layer4 = ConcatDifferentShapeBlocks::create(
                Box::new(layer4),
                Box::new(Or::<2>::new())
            );

            MergeLayers::create(Box::new(layer1), Box::new(layer2))
                .connect_to(Box::new(layer3))
                .connect_to(Box::new(layer4))
        };

        let comp_core = RecurrentBlock::<3, 2, 0, 8>::create_from_fn(make_comp);
        let zip_data = Wiring::<16, 16>::zip::<8>();
        let prev_input = ConcatBlocks::create(
            [Buffer::new(); 3].map(|c| Box::new(c) as Box<dyn Component<1, 1>>)
        );
        let layer1 = ConcatDifferentShapeBlocks::create(
            Box::new(prev_input),
            Box::new(zip_data)
        );
        let comp = MergeLayers::create(Box::new(layer1), Box::new(comp_core));

        Self { comp}
    }
}

#[test]
fn eight_bit_comparator_test() {
    use crate::num_bit_converter::*;
    let comp = EightBitComparator::new();
    for i in 0..256 {
        for j in 0..256 {
            let default = 2;
            let input = num_to_bit((((j << 8) + i) << 3) + default);
            assert_eq!(comp.eval(input), [i > j, i == j, i < j]);
        }
    }
}

struct EightBitMultiplier {
    multiplier: MergeLayers<16, 80, 16>,
}

impl Component<16, 16> for EightBitMultiplier {
    fn eval(&self, input: [bool; 16]) -> [bool; 16] {
        self.multiplier.eval(input)
    }
}

const BYTE: usize = 8;

impl EightBitMultiplier {
    fn dummy(num1: usize, num2: usize) -> usize {
        use crate::num_bit_converter::*;
        let num1_bit = num_to_bit::<BYTE>(num1);
        num1_bit.iter().enumerate()
            .map(|(i, &b)| if b {num2 << i} else {0})
            .sum()
    }
    fn new() -> Self {
        let one_bit_multiplier = || {
            // 先頭ビットは最後まで0
            let shifter = Wiring::<{2 * BYTE}, {2 * BYTE}>::rotate_right::<1>();
            let buffer = ConcatBlocks::create(
                [Buffer::new(); BYTE].map(|c| Box::new(c) as Box<dyn Component<1, 1>>)
            );
            let padding = Constant::<0, {2 * BYTE - BYTE}, false>::new();
            // let in_wrapper = Wiring::<8, 8>::wrapper();
            let cur_input = ConcatDifferentShapeBlocks::create(
                Box::new(buffer),
                Box::new(padding)
            );
            let all_input = ConcatDifferentShapeBlocks::create(
                Box::new(shifter),
                Box::new(cur_input)
            );
            let adder = NByteAdder::<2>::new();
            MergeLayers::create(Box::new(all_input), Box::new(adder))
        };

        let multiplier = RecurrentBlock::<{2 * BYTE}, 8, 0, BYTE>::create_from_fn(
            one_bit_multiplier
        );

        let rev = Wiring::<BYTE, BYTE>::reverse();
        let add_switch_branch = MergeLayers::<BYTE, BYTE, {BYTE * BYTE}>::create(
            Box::new(rev),
            Box::new(ConcatBlocks::<1, BYTE, BYTE>::create_from_fn(Branch::<BYTE>::new))
        );

        let multiplicand_branch = {
            let branch = ConcatBlocks::<1, BYTE, BYTE>::create_from_fn(Branch::<BYTE>::new);
            let unzip = Wiring::<{BYTE * BYTE}, {BYTE * BYTE}>::unzip::<BYTE>();
            MergeLayers::create(Box::new(branch), Box::new(unzip))
        };
        let zip_input_and_switch = {
            let c = ConcatBlocks::create(
                [Box::new(multiplicand_branch), Box::new(add_switch_branch)]
            );
            MergeLayers::create(
                Box::new(c),
                Box::new(Wiring::<{2 * BYTE * BYTE}, {2 * BYTE * BYTE}>::zip::<{BYTE * BYTE}>())
            )
        };
        let switched = MergeLayers::create(
            Box::new(zip_input_and_switch),
            Box::new(ConcatBlocks::<2, 1, {BYTE * BYTE}>::create_from_fn(And::<2>::new)),
        );
        let init_hidden_state = Constant::<0, {2 * BYTE}, false>::new();
        let input = ConcatDifferentShapeBlocks::create(
            Box::new(init_hidden_state),
            Box::new(switched)
        );
        let multiplier = MergeLayers::<{2 * BYTE}, {2 * BYTE + BYTE * BYTE}, {2 * BYTE}>::debug(
            Box::new(input),
            Box::new(multiplier)
        );

        Self {multiplier}
    }
}

#[test]
fn eight_bit_multiplier_test() {
    use crate::num_bit_converter::*;

    let mul = EightBitMultiplier::new();

    for i in 0..256 {
        for j in 0..256 {
            let input = num_to_bit((i << 8) + j);
            assert_eq!(bit_to_num(mul.eval(input)), i * j);
            assert_eq!(
                EightBitMultiplier::dummy(i, j),
                i * j
            );
        }
    }
}
