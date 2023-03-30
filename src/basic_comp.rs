use crate::core::*;

#[derive(Debug, Clone, Copy)]
pub struct Constant<const I: usize, const O: usize, const D: bool> {}
impl<const I: usize, const O: usize, const D: bool> Component<I, O> for Constant<I, O, D> {
    fn eval(&self, input: [bool; I]) -> [bool; O] {
        [D; O]
    }
}
impl<const I: usize, const O: usize, const D: bool> Constant<I, O, D> {
    pub fn new() -> Self {
        Self {}
    }
}

#[test]
fn false_test() {
    use crate::num_bit_converter::*;
    let f = Constant::<8, 8, false>::new();
    let t = Constant::<8, 8, true>::new();
    for i in 0..256 {
        assert_eq!(f.eval(num_to_bit::<8>(i)), [false; 8]);
        assert_eq!(t.eval(num_to_bit::<8>(i)), [true; 8]);
    }
}


#[derive(Debug, Clone, Copy)]
pub struct And<const I: usize> {}
impl<const I: usize> Component<I, 1> for And<I> {
    fn eval(&self, input: [bool; I]) -> [bool; 1] {
        [input.into_iter().all(|b| b)]
    }
}
impl<const I: usize> And<I> {
    pub fn new() -> Self {
        Self {}
    }
}

#[test]
fn and_test() {
    use crate::num_bit_converter::*;
    let c = And::<8>::new();
    for i in 0..255 {
        assert_eq!(c.eval(num_to_bit::<8>(i)), [false]);
    }
    assert_eq!(c.eval([true; 8]), [true]);
}

#[derive(Debug, Clone, Copy)]
pub struct Or<const I: usize> {}

impl<const I: usize> Component<I, 1> for Or<I> {
    fn eval(&self, input: [bool; I]) -> [bool; 1] {
        [input.into_iter().any(|b| b)]
    }
}
impl<const I: usize> Or<I> {
    pub fn new() -> Self {
        Self {}
    }
}

#[test]
fn or_test() {
    use crate::num_bit_converter::*;
    let c = Or::<8>::new();
    assert_eq!(c.eval([false; 8]), [false]);
    for i in 1..256 {
        assert_eq!(c.eval(num_to_bit::<8>(i)), [true]);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Not {}

impl Component<1, 1> for Not {
    fn eval(&self, input: [bool; 1]) -> [bool; 1] {
        [!input[0]]
    }
}
impl Not {
    pub fn new() -> Self {
        Self {}
    }
}

#[test]
fn not_test() {
    let c = Not::new();
    assert_eq!(c.eval([true]), [false]);
    assert_eq!(c.eval([false]), [true]);
}

#[derive(Debug, Clone, Copy)]
pub struct Buffer {}

impl Component<1, 1> for Buffer {
    fn eval(&self, input: [bool; 1]) -> [bool; 1] {
        [input[0]]
    }
}
impl Buffer {
    pub fn new() -> Self {
        Self {}
    }
}

#[test]
fn buffer_test() {
    let c = Buffer::new();
    assert_eq!(c.eval([true]), [true]);
    assert_eq!(c.eval([false]), [false]);
}

#[derive(Debug, Clone, Copy)]
pub struct Branch<const O: usize> {}

impl<const O: usize> Component<1, O> for Branch<O> {
    fn eval(&self, input: [bool; 1]) -> [bool; O] {
        [input[0]; O]
    }
}
impl<const O: usize> Branch<O> {
    pub fn new() -> Self {
        Self {}
    }
}

#[test]
fn branch_test() {
    let c = Branch::<8>::new();
    assert_eq!(c.eval([true]), [true; 8]);
    assert_eq!(c.eval([false]), [false; 8]);
}

pub struct NAND<const I: usize> {
    nand: MergeLayers<I, 1, 1>,
}
impl<const I: usize> Component<I, 1> for NAND<I> {
    fn eval(&self, input: [bool; I]) -> [bool; 1] {
        self.nand.eval(input)
    }
}
impl<const I: usize> NAND<I> {
    pub fn new() -> Self {
        Self {
            nand: MergeLayers::create(Box::new(And::<I>::new()), Box::new(Not::new())),
        }
    }
}

#[test]
fn nand_test() {
    use crate::num_bit_converter::*;
    let c = NAND::<8>::new();
    for i in 0..255 {
        assert_eq!(c.eval(num_to_bit::<8>(i)), [true]);
    }
    assert_eq!(c.eval([true; 8]), [false]);
}

pub struct XOR<const I: usize> {
    xor: MergeLayers<I, 2, 1>,
}
impl<const I: usize> Component<I, 1> for XOR<I> {
    fn eval(&self, input: [bool; I]) -> [bool; 1] {
        self.xor.eval(input)
    }
}
impl<const I: usize> XOR<I> 
where
    [(); I * 2]: Sized,
{
    pub fn new() -> Self {
        let mut layer1_table = [0; I * 2];
        for (i, v) in layer1_table.iter_mut().enumerate() {
            *v = i % I;
        }
        let layer1 = Wiring::create(layer1_table);
        let layer2 = ConcatBlocks::create(
            [Box::new(Or::<I>::new()), Box::new(NAND::<I>::new())]
        );
        let layer3 = And::<2>::new();
        let xor = MergeLayers::<I, {I * 2}, 2>::create(Box::new(layer1), Box::new(layer2))
            .connect_to(Box::new(layer3));
        Self {
            xor,
        }
    }
}

#[test]
fn xor_test() {
    use crate::num_bit_converter::*;
    let c = XOR::<8>::new();
    assert_eq!(c.eval([false; 8]), [false]);
    for i in 1..255 {
        assert_eq!(c.eval(num_to_bit::<8>(i)), [true]);
    }
    assert_eq!(c.eval([true; 8]), [false]);
}
