pub trait Component<const I: usize, const O: usize> {
    fn eval(&self, input: [bool; I]) -> [bool; O];
    // メモリなどで内部状態を変更しながら評価する
    fn eval_mut(&mut self, input: [bool; I]) -> [bool; O] {
        self.eval(input)
    }
}

pub struct DebugLayer<const N: usize> {}
impl<const N: usize> Component<N, N> for DebugLayer<N> {
    fn eval(&self, input: [bool; N]) -> [bool; N] {
        println!("{:?}", input);
        input
    }
}
impl<const N: usize> DebugLayer<N> {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct MergeLayers<const I: usize, const M: usize, const O: usize> {
    layer1: Box<dyn Component<I, M>>,
    layer2: Box<dyn Component<M, O>>,
}
impl<const I: usize, const M: usize, const O: usize> Component<I, O> for MergeLayers<I, M, O> {
    fn eval(&self, input: [bool; I]) -> [bool; O] {
        self.layer2.eval(self.layer1.eval(input))
    }
    fn eval_mut(&mut self, input: [bool; I]) -> [bool; O] {
        self.layer2.eval_mut(self.layer1.eval_mut(input))
    }
}
impl<const I: usize, const M: usize, const O: usize> MergeLayers<I, M, O> {
    pub fn create(layer1: Box<dyn Component<I, M>>, layer2: Box<dyn Component<M, O>>) -> Self {
        Self { layer1, layer2 }
    }
    pub fn debug(layer1: Box<dyn Component<I, M>>, layer2: Box<dyn Component<M, O>>) -> Self {
        let debug_layer = Box::new(DebugLayer::<M>::new());
        MergeLayers::create(layer1, debug_layer)
            .connect_to(layer2)
    }
    pub fn connect_to<const P: usize>(
        self,
        next_layer: Box<dyn Component<O, P>>,
    ) -> MergeLayers<I, O, P> {
        MergeLayers::create(Box::new(self), next_layer)
    }
    pub fn debug_connect<const P: usize>(
        self,
        next_layer: Box<dyn Component<O, P>>,
    ) -> MergeLayers<I, O, P> {
        let debug_layer = Box::new(DebugLayer::<O>::new());
        self.connect_to(debug_layer)
            .connect_to(next_layer)
    }
}

pub struct ConcatBlocks<const I: usize, const O: usize, const N: usize> {
    blocks: [Box<dyn Component<I, O>>; N],
}

impl<const I: usize, const O: usize, const N: usize> Component<{ I * N }, { O * N }>
    for ConcatBlocks<I, O, N>
where
    [(); I * N]: Sized,
    [(); O * N]: Sized,
{
    fn eval_mut(&mut self, input: [bool; I * N]) -> [bool; O * N] {
        let inputs = Self::split_input(input);
        let mut outputs = [[false; O]; N];
        for ((result, block), val) in outputs.iter_mut().zip(self.blocks.iter_mut()).zip(inputs) {
            result
                .iter_mut()
                .zip(block.eval_mut(val))
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
    pub fn create(blocks: [Box<dyn Component<I, O>>; N]) -> Self {
        Self { blocks }
    }
    pub fn create_from_fn<T>(f: fn() -> T) -> Self
    where
        T: Component<I, O> + Sized + 'static
    {
        let blocks = [0; N].map(|_| Box::new(f()) as Box<dyn Component<I, O>>);
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

pub struct ConcatDifferentShapeBlocks<
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
    fn eval_mut(&mut self, input: [bool; I1 + I2]) -> [bool; O1 + O2] {
        let (input1, input2) = Self::split_input(input);
        let output1 = self.block1.eval_mut(input1);
        let output2 = self.block2.eval_mut(input2);
        Self::merge_output(output1, output2)
    }
}
impl<const I1: usize, const I2: usize, const O1: usize, const O2: usize>
    ConcatDifferentShapeBlocks<I1, I2, O1, O2>
{
    pub fn create(block1: Box<dyn Component<I1, O1>>, block2: Box<dyn Component<I2, O2>>) -> Self {
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


#[derive(Debug)]
pub struct Wiring<const N: usize, const M: usize> {
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
    pub fn create(table: [usize; M]) -> Self {
        Self { table }
    }
}
impl<const N: usize, const M: usize> Wiring<N, M> {
    // コンパイラには違う型に見えるけど実際は同じものをラップする
    pub fn wrapper() -> Self {
        let mut table = [0; M];
        table.iter_mut().zip(0..N).for_each(|(t, n)| *t = n);
        Self { table }
    }
}
impl<const N: usize> Wiring<N, N> {
    pub fn zip<const S: usize>() -> Self {
        Self::unzip::<S>()
    }
    pub fn unzip<const S: usize>() -> Self {
        // [[usize; S]; sep] -> [[usize; sep]; S]
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
    pub fn zip_with_chunk<const S: usize>() -> Self {
        let mut table = [0; N];
        table.iter_mut().enumerate()
            .for_each(|(i, v)| {
                let chunk_index = i / (2 * S);
                let index_in_chunk = i % S;
                let is_in_block0 = i % (2 * S) < S;
                *v = chunk_index * S + index_in_chunk + if is_in_block0 {0} else {N / 2};
            });
        Self {table}
    }
    pub fn rotate_right<const S: usize>() -> Self {
        let mut table = [0; N];
        for (i, v) in table.iter_mut().enumerate() {
            *v = (i + N - S) % N;
        }
        Self {table}
    }
    pub fn reverse() -> Self {
        let mut table = [0; N];
        table.iter_mut().enumerate()
            .for_each(|(i, v)| *v = N - i - 1);
        Self {table}
    }
}

pub struct RecurrentBlock<const S: usize, const I: usize, const O: usize, const N: usize>
where
    [(); S + I]: Sized,
    [(); O + S]: Sized,
{
    blocks: [Box<dyn Component<{S + I}, {O + S}>>; N],
}

impl<
    const S: usize,
    const In: usize,
    const Out: usize,
    const N: usize,
> Component<{S + In * N}, {Out * N + S}> for RecurrentBlock<S, In, Out, N>
where
    [(); S + In]: Sized,
    [(); Out + S]: Sized,
    [(); S + In * N]: Sized,
    [(); Out * N + S]: Sized,
{
    fn eval(&self, input: [bool; S + In * N]) -> [bool; Out * N + S] {
        let mut acc = input[..S].to_vec();
        let mut result = [false; Out * N + S];
        for (i, block) in self.blocks.iter().enumerate() {
            let mut block_input = [false; S + In];
            block_input.iter_mut()
                .zip(acc.iter().chain(input[(S + i * In)..(S + (i + 1) * In)].iter()))
                .for_each(|(v1, v2)| *v1 = *v2);

            let block_output = block.eval(block_input);
            for j in 0..Out {
                result[Out * i + j] = block_output[j];
            }
            acc = block_output[Out..].to_vec();
        }

        for i in 0..S {
            result[Out * N + i] = acc[i];
        }

        result
    }
}

impl<const S: usize, const I: usize, const O: usize, const N: usize> RecurrentBlock<S, I, O, N>
where
    [(); S + I]: Sized,
    [(); O + S]: Sized,
    [(); S + I * N]: Sized,
    [(); O * N + S]: Sized,
{
    pub fn create(blocks: [Box<dyn Component<{S + I}, {O + S}>>; N]) -> Self {
        Self { blocks}
    }
    pub fn create_from_fn<T: Component<{S + I}, {O + S}> + Sized + 'static>(f: fn() -> T) -> Self {
        Self {
            blocks: [0; N].map(|_| Box::new(f()) as Box<dyn Component<{S+I}, {O+S}>>)
        }
    }
}

pub const fn pow2(n: usize) -> usize {
    2_usize.pow(n as u32)
}
