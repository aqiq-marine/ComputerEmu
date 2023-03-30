#![feature(generic_const_exprs)]

mod core;
use crate::core::*;
mod basic_comp;
mod memory;
use crate::memory::*;
mod decoder;
mod calculator;
use crate::calculator::*;
mod num_bit_converter;

fn main() {
    let mut adder = EightBitFullAdder::new();

    let mut memory = Memory::<10, 8>::new();
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
    let make_input = |n1: usize, n2: usize| -> [bool; 16] {
        let mut result = [false; 16];
        result.iter_mut()
            .zip(num_to_bit(n1).iter().chain(num_to_bit(n2).iter()))
            .for_each(|(v1, v2)| *v1 = *v2);
        result
    };
    let add = |adder: &mut EightBitFullAdder, n1: usize, n2: usize| -> usize {
        let input = make_input(n1, n2);
        let output = adder.eval(input);
        let mut floor_output = [false; 8];
        floor_output.iter_mut()
            .zip(output.iter())
            .for_each(|(v1, v2)| *v1 = *v2);
        bit_to_num(floor_output)
    };
    println!("{:?}", add(&mut adder, 2, 1));
    (0..256).for_each(|i| {
        (0..256).for_each(|j| {
            assert_eq!(add(&mut adder, i, j), (i + j) % 256)
        })
    });

    // let adder = FullAdder::new();
    // println!("{:?}", adder.eval([false, false, false]));

    // let adder = HalfAdder::new();
    // println!("{:?}", adder.eval([true, true]));

    // let create_input = |read: bool, write: bool, addr: usize, values: usize| -> [bool; 18] {
    //     let addr = num_to_bit(addr);
    //     let values = num_to_bit(values);
    //     let mut result = [false; 18];
    //     result.iter_mut()
    //         .zip([read, write].into_iter()
    //             .chain(addr.into_iter())
    //             .chain(values.into_iter()))
    //         .for_each(|(v1, v2)| *v1 = v2);
    //     result
    // };
    // let read = |memory: &mut Memory<8, 8>, addr: usize| -> usize {
    //     let input = create_input(true, false, addr, 0);
    //     let result = memory.eval_mut(input);
    //     bit_to_num(result)
    // };
    // let write = |memory: &mut Memory<8, 8>, addr: usize, val: usize| -> usize {
    //     let input = create_input(true, true, addr, val);
    //     let result = memory.memory.eval_mut(input);
    //     bit_to_num(result)
    // };

    // println!("{:?}", write(&mut memory, 1, 1));
    // for i in 0..128 {
    //     write(&mut memory, i, i);
    // }
    // for i in 128..256 {
    //     write(&mut memory, i, 0);
    // }
    // for i in 0..256 {
    //     println!("{:?}", read(&mut memory, i));
    // }

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
