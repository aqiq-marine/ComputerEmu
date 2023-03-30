use crate::core::*;

#[derive(Debug, Clone, Copy)]
pub struct False<const I: usize, const O: usize> {}
impl<const I: usize, const O: usize> Component<I, O> for False<I, O> {
    fn eval(&self, input: [bool; I]) -> [bool; O] {
        [false; O]
    }
}
impl<const I: usize, const O: usize> False<I, O> {
    pub fn new() -> Self {
        Self {}
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
