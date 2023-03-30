use crate::core::*;
use crate::basic_comp::*;

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
