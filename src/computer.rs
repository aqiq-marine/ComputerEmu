use crate::core::*;
use crate::basic_comp::*;
use crate::decoder::BitDecoder;
use crate::memory::*;
use crate::clock::*;
use crate::arithmetic_comp::*;

const MEMORY_ADDR_SIZE: usize = 16;

type MainMemory = Memory<MEMORY_ADDR_SIZE, 8>;

type ProgramCounter = MemoryByte<MEMORY_ADDR_SIZE>;
type AddressRegistar = [MemoryByte<MEMORY_ADDR_SIZE>; 8];
type GeneralRegistar = [MemoryByte<8>; 8];
type FlagRegistar = MemoryCell;

type InstDecoder = BitDecoder<8>;

struct Cpu {}

// struct AddValueToRegistar<const N: usize, const M: usize> {
// }
// 
// impl AddValueToRegistar<8, 8> {
//     fn new() -> Self {
//         let select = Buffer::new();
//         let cache = MemoryByte::<8>::new();
//         let read_registar = {
//             let mut blocks = [Constant::<0, 1, false>::new(); 10]
//                 .map(|c| Box::new(c) as Box<dyn Component<0, 1>>);
//             blocks[0] = Box::new(Constant::<0, 1, true>::new());
//             ConcatBlocks::create(blocks)
//         };
//         let addr = {
//         };
//         let make_command = {
//         };
//     }
// }

// impl AddValueToRegistar<8, 16> {
// }

struct AddRegistarToRegistar {}
struct AddRegistarToPointer {}
struct AddPointerToPointer {}


struct CopyValueToRegistar {}
struct CopyRegistarToRegistar {}
struct CopyRegistarToPointer {}
struct CopyPointerToPointer {}


struct Ite {}


struct JumpToRegistar {}
struct JumpToPointer {}


// 命令セット
// Add addr_addr value
// Add addr_addr addr_addr
// Add addr_addr general_addr
// Add general_addr general_addr
// Add general_addr value
// Add general_addr value (2 byte)
// mov addr_addr addr_addr
// mov general_addr addr_addr
// ite addr_addr addr_addr
// jmp addr_addr pad


struct DummyCell {
    s: bool
}

impl Component<3, 1> for DummyCell {
    fn eval(&self, input: [bool; 3]) -> [bool; 1] {
        println!("running immutable eval with input: {:?}", input);
        [self.s]
    }
    fn eval_mut(&mut self, input: [bool; 3]) -> [bool; 1] {
        println!("input is: {:?}", input);
        println!("inner state is: {}", self.s);
        if input[1] {
            self.s = input[2];
        }
        [self.s && input[0]]
    }
}
impl DummyCell {
    fn new() -> Self {
        Self {s: false}
    }
}

struct MicroProgramCounter<const N: usize>
where
    [(); 1 * N]: Sized,
{
    counter: MergeLayers<2, 2, N>,
}

impl<const N: usize> Component<2, N> for MicroProgramCounter<N>
where
    [(); 1 * N]: Sized,
{
    fn eval(&self, input: [bool; 2]) -> [bool; N] {
        self.counter.eval(input)
    }
    fn eval_mut(&mut self, input: [bool; 2]) -> [bool; N] {
        self.counter.eval_mut(input)
    }
}

impl<const N: usize> MicroProgramCounter<N>
where
    [(); 1 * N]: Sized,
    [(); 2 + 0 * N]: Sized,
    [(); 1 * N + 2]: Sized,
{
    fn new() -> Self {
        // clock wake, value (prev block)
        let create_block = || {
            let cell_input = ConcatDifferentShapeBlocks::create(
                Box::new(Constant::<0, 1, true>::new()),
                Box::new(Wiring::<2, 2>::buffer()),
            );
            let cell_input = MergeLayers::create(
                Box::new(cell_input),
                Box::new(Wiring::create([1, 0, 1, 2]))
            );
            let mut cell = ConcatDifferentShapeBlocks::create(
                Box::new(Buffer::new()),
                Box::new(MemoryCell::new())
            );
            {
                assert_eq!([false, false], cell.eval_mut([false, true, false, false]));
                assert_eq!([true, true], cell.eval_mut([true, true, true, true]));
                assert_eq!([false, true], cell.eval_mut([false, true, false, true]));

                assert_eq!([true, false], cell.eval_mut([true, true, true, false]));
            };
            let output_wiring = Wiring::create([1, 0, 1]);
            let output = ConcatBlocks::create(
                [Box::new(Buffer::new()),
                Box::new(Buffer::new()),
                Box::new(
                    MergeLayers::create(
                        Box::new(Not::new()),
                        Box::new(DetectClockWake::new()),
                    )
                )]
            );

            let mut block = MergeLayers::create(Box::new(cell_input), Box::new(cell))
                .connect_to(Box::new(output_wiring))
                .connect_to(Box::new(output));

            {
                assert_eq!([true, true, false], block.eval_mut([true, true]));
                assert_eq!([true, false, false], block.eval_mut([false, true]));
                assert_eq!([true, false, false], block.eval_mut([false, false]));
                assert_eq!([false, true, true], block.eval_mut([true, false]));
            };

            block
        };

        let counter = RecurrentBlock::<2, 0, 1, N>::create_from_fn(create_block);
        let input_wrapper = Wiring::<2, {2 + 0 * N}>::wrapper();
        let output_cut = Wiring::<{1 * N + 2}, N>::cut();

        let counter = MergeLayers::create(Box::new(input_wrapper), Box::new(counter))
            .connect_to(Box::new(output_cut));

        let clock_wake = ConcatBlocks::create([
            Box::new(DetectClockWake::new()),
            Box::new(Buffer::new())
        ]);

        let counter = MergeLayers::create(Box::new(clock_wake), Box::new(counter));
        Self { counter }
    }
}

#[test]
fn micro_program_counter_test() {
    let mut counter = MicroProgramCounter::<3>::new();
    assert_eq!([false, false, false], counter.eval_mut([false, false]));
    assert_eq!([false, false, false], counter.eval_mut([true, false]));
    assert_eq!([false, false, false], counter.eval_mut([false, true]));


    assert_eq!([true, false, false], counter.eval_mut([true, true]));
    assert_eq!([true, false, false], counter.eval_mut([false, true]));

    for i in 0..4 {
        let mut expect = [false; 3];
        if i + 1 < 3 {
            expect[i + 1] = true;
        }

        assert_eq!(counter.eval_mut([true, false]), expect);
        assert_eq!(counter.eval_mut([true, true]), expect);

        assert_eq!(counter.eval_mut([false, false]), expect);
        assert_eq!(counter.eval_mut([false, true]), expect);
    }
}
