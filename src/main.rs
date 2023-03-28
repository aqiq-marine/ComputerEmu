#![feature(generic_const_exprs)]

// use std::sync::mpsc::{Receiver, Sender, SendError, self};
// use rayon::prelude::*;

trait Component<const I: usize, const O: usize> {
    fn eval(&self, input: [bool; I]) -> [bool; O];
    fn eval_recur(&mut self, input: [bool; I]) -> [bool; O] {
        self.eval(input)
    }
}

struct DebugLayer<const N: usize> {}
impl<const N: usize> Component<N, N> for DebugLayer<N> {
    fn eval(&self, input: [bool; N]) -> [bool; N] {
        println!("{:?}", input);
        input
    }
}
impl<const N: usize> DebugLayer<N> {
    fn new() -> Self {
        Self {}
    }
}

struct MergeLayers<const I: usize, const M: usize, const O: usize> {
    layer1: Box<dyn Component<I, M>>,
    layer2: Box<dyn Component<M, O>>,
}
impl<const I: usize, const M: usize, const O: usize> Component<I, O> for MergeLayers<I, M, O> {
    fn eval(&self, input: [bool; I]) -> [bool; O] {
        self.layer2.eval(self.layer1.eval(input))
    }
    fn eval_recur(&mut self, input: [bool; I]) -> [bool; O] {
        self.layer2.eval_recur(self.layer1.eval_recur(input))
    }
}
impl<const I: usize, const M: usize, const O: usize> MergeLayers<I, M, O> {
    fn create(layer1: Box<dyn Component<I, M>>, layer2: Box<dyn Component<M, O>>) -> Self {
        Self { layer1, layer2 }
    }
    fn debug(layer1: Box<dyn Component<I, M>>, layer2: Box<dyn Component<M, O>>) -> Self {
        let debug_layer = Box::new(DebugLayer::<M>::new());
        MergeLayers::create(layer1, debug_layer)
            .connect_to(layer2)
    }
    fn connect_to<const P: usize>(
        self,
        next_layer: Box<dyn Component<O, P>>,
    ) -> MergeLayers<I, O, P> {
        MergeLayers::create(Box::new(self), next_layer)
    }
    fn debug_connect<const P: usize>(
        self,
        next_layer: Box<dyn Component<O, P>>,
    ) -> MergeLayers<I, O, P> {
        let debug_layer = Box::new(DebugLayer::<O>::new());
        self.connect_to(debug_layer)
            .connect_to(next_layer)
    }
}

struct ConcatBlocks<const I: usize, const O: usize, const N: usize> {
    blocks: [Box<dyn Component<I, O>>; N],
}

impl<const I: usize, const O: usize, const N: usize> Component<{ I * N }, { O * N }>
    for ConcatBlocks<I, O, N>
where
    [(); I * N]: Sized,
    [(); O * N]: Sized,
{
    fn eval_recur(&mut self, input: [bool; I * N]) -> [bool; O * N] {
        let inputs = Self::split_input(input);
        let mut outputs = [[false; O]; N];
        for ((result, block), val) in outputs.iter_mut().zip(self.blocks.iter_mut()).zip(inputs) {
            result
                .iter_mut()
                .zip(block.eval_recur(val))
                .for_each(|(v1, v2)| *v1 = v2);
        }
        Self::merge_output(outputs)
    }
    fn eval(&self, input: [bool; I * N]) -> [bool; O * N] {
        let inputs = Self::split_input(input);
        let mut outputs = [[false; O]; N];
        for ((result, block), val) in outputs.iter_mut().zip(self.blocks.iter()).zip(inputs) {
            result
                .iter_mut()
                .zip(block.eval(val))
                .for_each(|(v1, v2)| *v1 = v2);
        }
        Self::merge_output(outputs)
    }
}
impl<const I: usize, const O: usize, const N: usize> ConcatBlocks<I, O, N> {
    fn create(blocks: [Box<dyn Component<I, O>>; N]) -> Self {
        Self { blocks }
    }
    fn split_input(input: [bool; I * N]) -> [[bool; I]; N] {
        let mut inputs = [[false; I]; N];
        for (v1, v2) in inputs.iter_mut().flatten().zip(input) {
            *v1 = v2;
        }
        return inputs;
    }
    fn merge_output(outputs: [[bool; O]; N]) -> [bool; O * N] {
        let mut output = [false; O * N];
        for (v1, v2) in outputs.into_iter().flatten().zip(output.iter_mut()) {
            *v2 = v1;
        }
        return output;
    }
}

struct ConcatDifferentShapeBlocks<
    const I1: usize,
    const I2: usize,
    const O1: usize,
    const O2: usize,
> {
    block1: Box<dyn Component<I1, O1>>,
    block2: Box<dyn Component<I2, O2>>,
}

impl<const I1: usize, const I2: usize, const O1: usize, const O2: usize>
    Component<{ I1 + I2 }, { O1 + O2 }> for ConcatDifferentShapeBlocks<I1, I2, O1, O2>
{
    fn eval(&self, input: [bool; I1 + I2]) -> [bool; O1 + O2] {
        let (input1, input2) = Self::split_input(input);
        let output1 = self.block1.eval(input1);
        let output2 = self.block2.eval(input2);
        Self::merge_output(output1, output2)
    }
    fn eval_recur(&mut self, input: [bool; I1 + I2]) -> [bool; O1 + O2] {
        let (input1, input2) = Self::split_input(input);
        let output1 = self.block1.eval_recur(input1);
        let output2 = self.block2.eval_recur(input2);
        Self::merge_output(output1, output2)
    }
}
impl<const I1: usize, const I2: usize, const O1: usize, const O2: usize>
    ConcatDifferentShapeBlocks<I1, I2, O1, O2>
{
    fn create(block1: Box<dyn Component<I1, O1>>, block2: Box<dyn Component<I2, O2>>) -> Self {
        Self { block1, block2 }
    }
    fn split_input(input: [bool; I1 + I2]) -> ([bool; I1], [bool; I2]) {
        let mut input1 = [false; I1];
        let mut input2 = [false; I2];
        for (v1, v2) in input1.iter_mut().chain(input2.iter_mut()).zip(input) {
            *v1 = v2;
        }
        return (input1, input2);
    }
    fn merge_output(output1: [bool; O1], output2: [bool; O2]) -> [bool; O1 + O2] {
        let mut output = [false; O1 + O2];
        let output_chain = output1.into_iter().chain(output2);
        for (v1, v2) in output1.into_iter().chain(output2).zip(output.iter_mut()) {
            *v2 = v1;
        }
        return output;
    }
}

#[derive(Debug, Clone, Copy)]
struct False<const I: usize, const O: usize> {}
impl<const I: usize, const O: usize> Component<I, O> for False<I, O> {
    fn eval(&self, input: [bool; I]) -> [bool; O] {
        [false; O]
    }
}
impl<const I: usize, const O: usize> False<I, O> {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy)]
struct And<const I: usize> {}
impl<const I: usize> Component<I, 1> for And<I> {
    fn eval(&self, input: [bool; I]) -> [bool; 1] {
        [input.into_iter().all(|b| b)]
    }
}
impl<const I: usize> And<I> {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy)]
struct Or<const I: usize> {}

impl<const I: usize> Component<I, 1> for Or<I> {
    fn eval(&self, input: [bool; I]) -> [bool; 1] {
        [input.into_iter().any(|b| b)]
    }
}
impl<const I: usize> Or<I> {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy)]
struct Not {}

impl Component<1, 1> for Not {
    fn eval(&self, input: [bool; 1]) -> [bool; 1] {
        [!input[0]]
    }
}
impl Not {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy)]
struct Buffer {}

impl Component<1, 1> for Buffer {
    fn eval(&self, input: [bool; 1]) -> [bool; 1] {
        [input[0]]
    }
}
impl Buffer {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy)]
struct Branch<const O: usize> {}

impl<const O: usize> Component<1, O> for Branch<O> {
    fn eval(&self, input: [bool; 1]) -> [bool; O] {
        [input[0]; O]
    }
}
impl<const O: usize> Branch<O> {
    fn new() -> Self {
        Self {}
    }
}

struct NAND<const I: usize> {
    nand: MergeLayers<I, 1, 1>,
}
impl<const I: usize> Component<I, 1> for NAND<I> {
    fn eval(&self, input: [bool; I]) -> [bool; 1] {
        self.nand.eval(input)
    }
}
impl<const I: usize> NAND<I> {
    fn new() -> Self {
        Self {
            nand: MergeLayers::create(Box::new(And::<I>::new()), Box::new(Not::new())),
        }
    }
}


// Reset, Setの順
struct RSFlipFlop {
    ff: MergeLayers<4, 4, 2>,
    nand1_to_nand2_line_state: bool,
    nand2_to_rand1_line_state: bool,
}

impl Component<2, 2> for RSFlipFlop {
    fn eval(&self, input: [bool; 2]) -> [bool; 2] {
        self.ff.eval(self.input_with_cache(input))
    }
    fn eval_recur(&mut self, input: [bool; 2]) -> [bool; 2] {
        for _ in 0..8 {
            let result = self.ff.eval_recur(self.input_with_cache(input));
            self.nand1_to_nand2_line_state = result[0];
            self.nand2_to_rand1_line_state = result[1];
        }
        self.eval(input)
    }
}

impl RSFlipFlop {
    fn new() -> Self {
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

#[derive(Debug)]
struct Wiring<const N: usize, const M: usize> {
    table: [usize; M],
}
impl<const N: usize, const M: usize> Component<N, M> for Wiring<N, M> {
    fn eval(&self, input: [bool; N]) -> [bool; M] {
        let mut output = [false; M];
        for (v, i) in output.iter_mut().zip(self.table) {
            *v = input[i];
        }
        return output;
    }
}
impl<const N: usize, const M: usize> Wiring<N, M> {
    fn create(table: [usize; M]) -> Self {
        Self { table }
    }
}
impl<const N: usize, const M: usize> Wiring<N, M> {
    // コンパイラには違う型に見えるけど実際は同じものをラップする
    fn wrapper() -> Self {
        let mut table = [0; M];
        table.iter_mut().zip(0..N).for_each(|(t, n)| *t = n);
        Self { table }
    }
}
impl<const N: usize> Wiring<N, N> {
    fn unzip<const S: usize>() -> Self {
        // [[usize; S]; sep] -> [[usize; sep];S]
        // branchなどで固まっているものをばらす
        let mut table = [0; N];
        let sep = N / S;
        for i in 0..S {
            for j in 0..sep {
                table[i * sep + j] = j * S + i;
            }
        }
        Self { table }
    }
}

const fn pow2(n: usize) -> usize {
    2_usize.pow(n as u32)
}

struct BitDecoder<const N: usize>
where
    MergeLayers<N, { N * pow2(N) }, { pow2(N) }>: Sized,
{
    decoder: MergeLayers<N, { N * pow2(N) }, { pow2(N) }>,
}

impl<const N: usize> Component<N, { pow2(N) }> for BitDecoder<N>
where
    MergeLayers<N, { N * pow2(N) }, { pow2(N) }>: Sized,
{
    fn eval(&self, input: [bool; N]) -> [bool; pow2(N)] {
        self.decoder.eval(input)
    }
}
impl<const N: usize> BitDecoder<N>
where
    MergeLayers<N, { 2_usize * N }, { pow2(N) }>: Sized,
    [(); 1 * N]: Sized,
    [(); 2 * N]: Sized,
    [(); 1 * N * 2]: Sized,
    [(); pow2(N - 1)]: Sized,
    [(); 1 * pow2(N)]: Sized,
    [(); N * pow2(N)]: Sized,
    Box<dyn Component<{ 2 * 1 * N }, { 2 * 1 * N }>>: Sized,
{
    fn new() -> Self {
        let layer1 = ConcatBlocks::create(
            [Branch::<2>::new(); N].map(|b| Box::new(b) as Box<dyn Component<1, 2>>),
        );
        let layer2 = Wiring::<{ 2 * N }, { 2 * N }>::unzip::<2>();

        let layer3: ConcatBlocks<{ 1 * N }, { 1 * N }, 2> = {
            let not = Box::new(ConcatBlocks::create(
                [Not::new(); N].map(|n| Box::new(n) as Box<dyn Component<1, 1>>),
            )) as Box<dyn Component<{ 1 * N }, { 1 * N }>>;

            let buffer = Box::new(ConcatBlocks::create(
                [Buffer::new(); N].map(|n| Box::new(n) as Box<dyn Component<1, 1>>),
            )) as Box<dyn Component<{ 1 * N }, { 1 * N }>>;

            ConcatBlocks::create([not, buffer])
        };
        // layer3終了時点でnot 1, not 2, ..., not N, 1, 2, ..., N

        let layer4 = {
            let mut table = [0; N * pow2(N)];
            // N本のinputからnotかbufferを選んでN本を構成する
            // N本（Andの入力）を2^N組（N bitのすべてのパターン）つくる
            for i in 0..pow2(N) {
                for j in 0..N {
                    // N * i + j番地を考える
                    // iのj bit目が0であればnotのj番につなぐ
                    let not_or_buffer = if i & (1 << j) == 0 { 0 } else { 1 };
                    let addr = N * i + j;
                    table[addr] = N * not_or_buffer + j;
                }
            }
            Wiring::create(table)
        };

        let layer5 = ConcatBlocks::create(
            [And::<N>::new(); pow2(N)].map(|a| Box::new(a) as Box<dyn Component<N, 1>>),
        );

        let in_wrapper =
            Box::new(Wiring::<N, { 1 * N }>::wrapper()) as Box<dyn Component<N, { 1 * N }>>;
        let layer1 = Box::new(layer1) as Box<dyn Component<{ 1 * N }, { 2 * N }>>;
        let layer2 = Box::new(layer2) as Box<dyn Component<{ 2 * N }, { 2 * N }>>;
        let layer23_wrapper = Box::new(Wiring::<{ 2 * N }, { 1 * N * 2 }>::wrapper())
            as Box<dyn Component<{ 2 * N }, { 1 * N * 2 }>>;
        let layer3 = Box::new(layer3) as Box<dyn Component<{ 1 * N * 2 }, { 1 * N * 2 }>>;
        let layer4 = Box::new(layer4) as Box<dyn Component<{ 1 * N * 2 }, { N * pow2(N) }>>;
        let layer5 = Box::new(layer5) as Box<dyn Component<{ N * pow2(N) }, { 1 * pow2(N) }>>;
        let out_wrapper = Box::new(Wiring::<{ 1 * pow2(N) }, { pow2(N) }>::wrapper())
            as Box<dyn Component<{ 1 * pow2(N) }, { pow2(N) }>>;

        let layer5 = Box::new(MergeLayers::create(layer5, out_wrapper))
            as Box<dyn Component<{ N * pow2(N) }, { pow2(N) }>>;

        let decoder = MergeLayers::create(in_wrapper, layer1)
            .connect_to(layer2)
            .connect_to(layer23_wrapper)
            .connect_to(layer3)
            .connect_to(layer4)
            .connect_to(layer5);
        Self { decoder }
    }
}

struct MemoryCell {
    cell: MergeLayers<3, 2, 1>,
}

impl Component<3, 1> for MemoryCell {
    fn eval_recur(&mut self, input: [bool; 3]) -> [bool; 1] {
        self.cell.eval_recur(input)
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

struct MemoryByte<const N: usize> where
    [(); N + 2]: Sized,
    [(); 3 * N]: Sized,
{
    byte: MergeLayers<{N + 2}, {3 * N}, N>
}

impl<const N: usize> Component<{N + 2}, N> for MemoryByte<N> where
    [(); 3 * N]: Sized,
{
    fn eval_recur(&mut self, input: [bool; N + 2]) -> [bool; N] {
        self.byte.eval_recur(input)
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
    fn new() -> Self {
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

struct Memory<const Address: usize, const Bit: usize> where
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
    fn eval_recur(&mut self, input: [bool; Address + Bit + 2]) -> [bool; Bit] {
        self.memory.eval_recur(input)
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
    fn new() -> Self {
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

        let mut memory = MergeLayers::create(Box::new(in_wrapper), Box::new(layer1))
            .connect_to(Box::new(layer2))
            .connect_to(Box::new(bytes))
            .connect_to(Box::new(layer3))
            .connect_to(Box::new(layer34_wrapper))
            .connect_to(Box::new(layer4));

        Self {memory}
    }
}

struct HalfAdder {
    adder: MergeLayers<2, 4, 2>,
}
impl Component<2, 2> for HalfAdder {
    fn eval(&self, input: [bool; 2]) -> [bool; 2] {
        self.adder.eval(input)
    }
}

impl HalfAdder {
    fn new() -> Self {
        let layer1 = Wiring::create([0, 1, 0, 1]);
        let layer2 = ConcatBlocks::create(
            [And::<2>::new(); 2].map(|a| Box::new(a) as Box<dyn Component<2, 1>>)
        );
        let adder = MergeLayers::create(Box::new(layer1), Box::new(layer2));
        Self {adder}
    }
}

struct FullAdder {
    adder: MergeLayers<3, 3, 2>,
}

impl Component<3, 2> for FullAdder {
    fn eval(&self, input: [bool; 3]) -> [bool; 2] {
        self.adder.eval(input)
    }
}

impl FullAdder {
    fn new() -> Self {
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


fn main() {
    let mut memory = Memory::<8, 8>::new();
    let bit_to_num = |bits: [bool; 8]| -> usize {
        bits.iter().enumerate()
            .map(|(i, &b)| (1 << i) * if b {1} else {0})
            .sum()
    };
    let num_to_bit = |num: usize| -> [bool; 8] {
        let mut bits = [false; 8];
        bits.iter_mut().enumerate()
            .for_each(|(i, b)| *b = num & (1 << i) != 0);
        bits
    };
    let create_input = |read: bool, write: bool, addr: usize, values: usize| -> [bool; 18] {
        let addr = num_to_bit(addr);
        let values = num_to_bit(values);
        let mut result = [false; 18];
        result.iter_mut()
            .zip([read, write].into_iter()
                .chain(addr.into_iter())
                .chain(values.into_iter()))
            .for_each(|(v1, v2)| *v1 = v2);
        result
    };
    let read = |memory: &mut Memory<8, 8>, addr: usize| -> usize {
        let input = create_input(true, false, addr, 0);
        let result = memory.eval_recur(input);
        bit_to_num(result)
    };
    let write = |memory: &mut Memory<8, 8>, addr: usize, val: usize| -> usize {
        let input = create_input(true, true, addr, val);
        let result = memory.memory.eval_recur(input);
        bit_to_num(result)
    };

    // println!("{:?}", write(&mut memory, 1, 1));
    for i in 0..128 {
        write(&mut memory, i, i);
    }
    for i in 128..256 {
        write(&mut memory, i, 0);
    }
    for i in 0..256 {
        println!("{:?}", read(&mut memory, i));
    }

    // let mut ff = RSFlipFlop::new();
    // println!("{:?}", ff.eval_recur([false, true]));

    // let mut cell = MemoryCell::new();
    // println!("{:?}", cell.eval_recur([true, true, false]));
    // println!("{:?}", cell.eval_recur([true, true, true]));
    // println!("{:?}", cell.eval_recur([true, false, false]));
    // println!("{:?}", cell.eval_recur([false, false, false]));
    
    // let mut ff = MemoryByte::<8>::new();
    // let rw_input = |n: usize| -> [bool; 10] {
    //     let bits = num_to_bit(n);
    //     let mut result = [false; 10];
    //     for i in 0..8 {
    //         result[i + 2] = bits[i];
    //     }
    //     result[0] = true;
    //     result[1] = true;
    //     result
    // };
    // println!("{:?}", bit_to_num(ff.eval_recur(rw_input(127))));

    // let mut decoder = BitDecoder::<8>::new();
    // println!("{:?}", decoder.eval_recur([true; 8]));
}
