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


pub struct EightBitFullAdder {
    adder: MergeLayers<16, 17, 9>,
}
impl Component<16, 9> for EightBitFullAdder {
    fn eval(&self, input: [bool; 16]) -> [bool; 9] {
        self.adder.eval(input)
    }
}
impl EightBitFullAdder {
    pub fn new() -> Self {
        let mut layer1_table = [0; 17];
        layer1_table.iter_mut().enumerate()
            .for_each(|(i, v)| *v = i.max(1) - 1);
        let layer1 = Wiring::create(layer1_table);
        let layer2 = Wiring::<16, 16>::zip::<8>();
        let layer2 = ConcatDifferentShapeBlocks::create(
            Box::new(False::<1, 1>::new()), Box::new(layer2)
        );
        let eight_bit_adder = RecurrentBlock::<1, 2, 1, 8>::create_from_fn(FullAdder::new);
        let adder = MergeLayers::create(Box::new(layer1), Box::new(layer2))
            .connect_to(Box::new(eight_bit_adder));
        Self {adder}
    }
}
