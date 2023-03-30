use crate::core::*;
use crate::basic_comp::*;
use crate::decoder::*;

// Reset, Setの順
pub struct RSFlipFlop {
    ff: MergeLayers<4, 4, 2>,
    nand1_to_nand2_line_state: bool,
    nand2_to_rand1_line_state: bool,
}

impl Component<2, 2> for RSFlipFlop {
    fn eval(&self, input: [bool; 2]) -> [bool; 2] {
        self.ff.eval(self.input_with_cache(input))
    }
    fn eval_mut(&mut self, input: [bool; 2]) -> [bool; 2] {
        for _ in 0..8 {
            let result = self.ff.eval_mut(self.input_with_cache(input));
            self.nand1_to_nand2_line_state = result[0];
            self.nand2_to_rand1_line_state = result[1];
        }
        self.eval(input)
    }
}

impl RSFlipFlop {
    pub fn new() -> Self {
        // Reset, Setの順にする
        let in_wrapper = Wiring::create([3, 1, 2, 0]);
        let layer1 = ConcatBlocks::create([
            Box::new(Not::new()),
            Box::new(Buffer::new()),
            Box::new(Buffer::new()),
            Box::new(Not::new()),
        ]);
        let layer2 = ConcatBlocks::create([Box::new(NAND::<2>::new()), Box::new(NAND::<2>::new())]);
        let ff = MergeLayers::create(Box::new(in_wrapper), Box::new(layer1))
            .connect_to(Box::new(layer2));
        Self {
            ff,
            nand2_to_rand1_line_state: false,
            nand1_to_nand2_line_state: false,
        }
    }
    fn input_with_cache(&self, input: [bool; 2]) -> [bool; 4] {
        [
            input[0],
            self.nand2_to_rand1_line_state,
            self.nand1_to_nand2_line_state,
            input[1],
        ]
    }
}

#[test]
fn rsflipflop_test() {
    let mut ff = RSFlipFlop::new();

    // 内部状態: [false, false]
    assert_eq!(ff.eval_mut([true, false]), [false, true]);
    // [false, true]
    assert_eq!(ff.eval_mut([true, false]), [false, true]);
    assert_eq!(ff.eval_mut([false, false]), [false, true]);
    assert_eq!(ff.eval_mut([false, true]), [true, false]);
    // [true, false]
    assert_eq!(ff.eval_mut([false, false]), [true, false]);
    assert_eq!(ff.eval_mut([false, true]), [true, false]);
    assert_eq!(ff.eval_mut([true, false]), [false, true]);
}


struct MemoryCell {
    cell: MergeLayers<3, 2, 1>,
}

impl Component<3, 1> for MemoryCell {
    fn eval_mut(&mut self, input: [bool; 3]) -> [bool; 1] {
        self.cell.eval_mut(input)
    }
    fn eval(&self, input: [bool; 3]) -> [bool; 1] {
        self.cell.eval(input)
    }
}

impl MemoryCell {
    fn new() -> Self {
        let cell: MergeLayers<2, 2, 1> = {
            let layer1 = Wiring::<2, 4>::create([0, 1, 0, 1]);
            let layer2 = ConcatBlocks::create(
                [Box::new(Buffer::new()) as Box<dyn Component<1, 1>>,
                Box::new(Not::new()) as Box<dyn Component<1, 1>>,
                Box::new(Buffer::new()) as Box<dyn Component<1, 1>>,
                Box::new(Buffer::new()) as Box<dyn Component<1, 1>>]
            );
            let layer3 = ConcatBlocks::create(
                [And::<2>::new(); 2].map(|a| Box::new(a) as Box<dyn Component<2, 1>>)
            );
            let ff = RSFlipFlop::new();
            let layer1 = Box::new(layer1);
            let layer2 = Box::new(layer2);
            let layer3 = Box::new(layer3);
            let layer4 = Box::new(ff) as Box<dyn Component<2, 2>>;
            let pick_only_q = Box::new(Wiring::create([0]));
            
            MergeLayers::create(layer1, layer2)
                .connect_to(layer3)
                .connect_to(layer4)
                .connect_to(pick_only_q)
        };
        let read_select = And::<2>::new();
        let cell = ConcatDifferentShapeBlocks::<1, 2, 1, 1>::create(
            Box::new(Buffer::new()),
            Box::new(cell)
        );
        let cell = MergeLayers::create(Box::new(cell), Box::new(read_select));
        Self {cell}
    }
}

#[test]
fn memory_cell_test() {
    use crate::num_bit_converter::num_to_bit;

    let mut cell = MemoryCell::new();
    let mut state = cell.eval_mut([true, true, false])[0];
    for i in 0..8 {
        let input = num_to_bit::<3>(i);
        if input[1] {
            state = input[2];
        }
        assert_eq!(cell.eval_mut(input), [input[0] && state]);
    }
}

pub struct MemoryByte<const N: usize> where
    [(); N + 2]: Sized,
    [(); 3 * N]: Sized,
{
    byte: MergeLayers<{N + 2}, {3 * N}, N>
}

impl<const N: usize> Component<{N + 2}, N> for MemoryByte<N> where
    [(); 3 * N]: Sized,
{
    fn eval_mut(&mut self, input: [bool; N + 2]) -> [bool; N] {
        self.byte.eval_mut(input)
    }
    fn eval(&self, input: [bool; N + 2]) -> [bool; N] {
        self.byte.eval(input)
    }
}

impl<const N: usize> MemoryByte<N> where
    [(); N + 2]: Sized,
    [(); 1 * N]: Sized,
    [(); 3 * N]: Sized,
{
    pub fn new() -> Self {
        let mut layer1_table = [0; 3 * N];
        layer1_table.iter_mut()
            .enumerate()
            .for_each(|(i, v)| *v = match i % 3 {
                0 | 1 => i % 3,
                _ => i / 3 + 2,
            });
        let layer1: Wiring<{N + 2}, {3 * N}> = Wiring::create(layer1_table);
        let cells = ConcatBlocks::create(
            [0; N].map(|c| Box::new(MemoryCell::new()) as Box<dyn Component<3, 1>>)
        );

        let out_wrapper = Wiring::<{1 * N}, N>::wrapper();
        let cells = MergeLayers::create(Box::new(cells), Box::new(out_wrapper));

        let byte: MergeLayers<{N + 2}, {3 * N}, N> = MergeLayers::create(
            Box::new(layer1),
            Box::new(cells)
        );
        Self {byte}
    }

}

macro_rules! ExistUntilx4 {
    ($N:expr) => {
        (
            [(); $N],
            [(); 1 * $N],
            [(); 2 * $N],
            [(); 3 * $N],
            [(); 4 * $N],
            [(); 1 * (1 * $N)],
            [(); 1 * (2 * $N)],
            [(); 1 * (3 * $N)],
            [(); 1 * (4 * $N)],
            [(); 2 * (1 * $N)],
            [(); 2 * (2 * $N)],
        )
    };
}

#[test]
fn memory_byte_test() {
    use crate::num_bit_converter::*;

    let mut byte = MemoryByte::<8>::new();
    let read_flag = 1;
    let write_flag = 2;
    for i in 0..256 {
        let num = 255 - i;
        assert_eq!(
            byte.eval_mut(num_to_bit::<10>((num << 2) + write_flag)),
            [false; 8]
        );
        assert_eq!(
            byte.eval_mut(num_to_bit::<10>(read_flag)),
            num_to_bit(num)
        );
    }
    for i in 0..256 {
        assert_eq!(
            byte.eval_mut(num_to_bit::<10>((i << 2) + write_flag + read_flag)),
            num_to_bit::<8>(i)
        );
        assert_eq!(
            byte.eval_mut(num_to_bit::<10>(read_flag)),
            num_to_bit::<8>(i)
        );
    }
}

pub struct Memory<const Address: usize, const Bit: usize> where
    [(); Address + Bit + 2]: Sized,
    [(); pow2(Address) * Bit]: Sized,
{
    memory: MergeLayers<{Address + Bit + 2}, {pow2(Address) * Bit}, Bit>
}

impl<const Address: usize, const Bit: usize>
    Component<{Address + Bit + 2}, Bit> for Memory<Address, Bit>
where
    [(); Address + Bit + 2]: Sized,
    [(); pow2(Address) * Bit]: Sized,
{
    fn eval_mut(&mut self, input: [bool; Address + Bit + 2]) -> [bool; Bit] {
        self.memory.eval_mut(input)
    }
    fn eval(&self, input: [bool; Address + Bit + 2]) -> [bool; Bit] {
        self.memory.eval(input)
    }
}


impl<const Address: usize, const Bit: usize> Memory<Address, Bit> where
    [(); Address + Bit + 2]: Sized,
    ([(); 2 + Address], [(); 2 + Address + 1 * Bit]): Sized,
    [(); pow2(Address) * Bit]: Sized,
    [(); Bit * pow2(Address)]: Sized,
    [(); (Bit + 2) * pow2(Address)]: Sized,
    [(); 2 * pow2(Address) + 1 * Bit]: Sized,
    [(); 2 * pow2(Address) * Bit]: Sized,
    [(); 2 + pow2(Address)]: Sized,
    ExistUntilx4!(pow2(Address)): Sized,
    ExistUntilx4!(Bit): Sized,

    BitDecoder<Address>: Sized,
    MergeLayers<Address, { 2_usize * Address }, { pow2(Address) }>: Sized,
    [(); 1 * Address * 2]: Sized,
    [(); pow2(Address - 1)]: Sized,
    Or<{pow2(Address)}>: Sized,
    [(); Address * pow2(Address)]: Sized,
    Box<dyn Component<{ 2 * 1 * Address }, { 2 * 1 * Address }>>: Sized,
{
    pub fn new() -> Self {
        // 2 (read, write) * 2^Address (バイト数)
        let read_write_addr_select: MergeLayers<{2 + Address}, {4 * pow2(Address)}, {2 * pow2(Address)}> = {
            let decoder = BitDecoder::<Address>::new();
            let read_write = ConcatBlocks::create(
                [Buffer::new(); 2].map(|b| Box::new(b) as Box<dyn Component<1, 1>>)
            );
            let layer1 = ConcatDifferentShapeBlocks::<2, Address, 2, {pow2(Address)}>::create(
                Box::new(read_write),
                Box::new(decoder)
            );
            let mut layer2_table = [0; 4 * pow2(Address)];
            layer2_table.iter_mut()
                .enumerate()
                .for_each(|(i, v)| *v = match i % 4 {
                    0 => 0,
                    2 => 1,
                    _ => i / 4 + 2,
                });
            let layer2 = Wiring::<{2 + pow2(Address)}, {4 * pow2(Address)}>::create(layer2_table);
            let layer23_wrapper = Wiring::<{4 * pow2(Address)}, {2 * (2 * pow2(Address))}>::wrapper();
            let layer3 = ConcatBlocks::<2, 1, {2 * pow2(Address)}>::create(
                [And::<2>::new(); 2 * pow2(Address)].map(|b| Box::new(b) as Box<dyn Component<2, 1>>)
            );
            let out_wapper = Wiring::<{1 * (2 * pow2(Address))}, {2 * pow2(Address)}>::wrapper();

            let layer3 = MergeLayers::create(Box::new(layer23_wrapper), Box::new(layer3))
                .connect_to(Box::new(out_wapper));

            MergeLayers::create(Box::new(layer1), Box::new(layer2))
                .connect_to(Box::new(layer3))
        };

        let values = ConcatBlocks::create(
            [Buffer::new(); Bit].map(|b| Box::new(b) as Box<dyn Component<1, 1>>)
        );

        let in_wrapper = Wiring::<{Address + Bit + 2}, {2 + Address + 1 * Bit}>::wrapper();
        let layer1 = ConcatDifferentShapeBlocks::<{2 + Address}, {1 * Bit}, {2 * pow2(Address)}, {1 * Bit}>::create(
            Box::new(read_write_addr_select),
            Box::new(values)
        );
        let mut layer2_table = [0; (Bit + 2) * pow2(Address)];
        layer2_table.iter_mut()
            .enumerate()
            .for_each(|(i, v)| {
                let p = i % (Bit + 2);
                *v = if p == 0 || p == 1 {
                    2 * i / (Bit + 2) + p
                } else {
                    // アドレスごとのread, writeの読み飛ばし + p - 2 (read, write)
                    2 * pow2(Address) + p - 2
                }
            });
        // read0, write0, read1, write1, ...
        // -> read0, data0, data1, ..., data_{Bit}, write0, data0, ..., data_{Bit}, ...
        let layer2 = Wiring::<{2 * pow2(Address) + 1 * Bit}, {(Bit + 2) * pow2(Address)}>::create(layer2_table);
        let bytes = ConcatBlocks::<{Bit + 2}, Bit, {pow2(Address)}>::create(
            [0; pow2(Address)].map(|_| {
                Box::new(MemoryByte::new()) as Box<dyn Component<{Bit + 2}, Bit>>
            })
        );
        let layer3 = Wiring::<{Bit * pow2(Address)}, {Bit * pow2(Address)}>::unzip::<Bit>();
        let layer34_wrapper = Wiring::<{Bit * pow2(Address)}, {pow2(Address) * Bit}>::wrapper();
        let layer4 = ConcatBlocks::create(
            [Or::<{pow2(Address)}>::new(); Bit].map(|c| {
                Box::new(c) as Box<dyn Component<{pow2(Address)}, 1>>
            })
        );
        let out_wrapper = Wiring::<{1 * Bit}, Bit>::wrapper();
        let layer4 = MergeLayers::create(Box::new(layer4), Box::new(out_wrapper));

        let memory = MergeLayers::create(Box::new(in_wrapper), Box::new(layer1))
            .connect_to(Box::new(layer2))
            .connect_to(Box::new(bytes))
            .connect_to(Box::new(layer3))
            .connect_to(Box::new(layer34_wrapper))
            .connect_to(Box::new(layer4));

        Self {memory}
    }
}

#[test]
fn memory_test() {
    use crate::num_bit_converter::*;

    let mut memory = Memory::<8, 8>::new();

    for i in 0..256 {
        let read = 1;
        let write = 2;
        let addr = i;
        let num = 255 - i;
        let input = num_to_bit::<18>((num << 10) + (addr << 2) + write + read);
        assert_eq!(memory.eval_mut(input), num_to_bit(num));
    }
}
