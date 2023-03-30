use crate::core::*;
use crate::basic_comp::*;

pub struct BitDecoder<const N: usize>
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
    pub fn new() -> Self {
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

#[test]
fn decoder_test() {
    use crate::num_bit_converter::*;
    let decoder = BitDecoder::<8>::new();
    for i in 0..256 {
        let expected_output = {
            let mut ex = [false; 256];
            ex[i] = true;
            ex
        };
        assert_eq!(decoder.eval(num_to_bit::<8>(i)), expected_output);
    }
}
