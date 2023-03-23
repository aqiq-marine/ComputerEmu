use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, SendError, self};
use rayon::prelude::*;

trait Component {
    fn recv(&mut self);
    fn send(&self) -> Result<(), SendError<bool>>;
    fn fast_send(&mut self) -> Result<(), SendError<bool>> {
        self.send()
    }
    fn step(&mut self) -> Result<(), SendError<bool>> {
        self.recv();
        self.send()
    }
    fn needed_step(&self) -> usize {
        1
    }
}
#[derive(Debug)]
struct And {
    cache: bool,
    input1: Receiver<bool>,
    input2: Receiver<bool>,
    output: Sender<bool>,
}

impl Component for And {
    fn recv(&mut self) {
        let input1 = self.input1.recv().unwrap_or(false);
        let input2 = self.input2.recv().unwrap_or(false);
        self.cache = input1 && input2;
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.output.send(self.cache)
    }
}
impl And {
    fn new(
        input1: Receiver<bool>,
        input2: Receiver<bool>,
        output: Sender<bool>,
    ) -> Self {
        Self {cache: false, input1, input2, output}
    }
    fn create(input1: Receiver<bool>, input2: Receiver<bool>) -> (Self, Receiver<bool>) {
        let (s, r) = mpsc::channel();
        (Self::new(input1, input2, s), r)
    }
}

#[derive(Debug)]
struct AndN {
    cache: bool,
    input: Vec<Receiver<bool>>,
    output: Sender<bool>,
}
impl Component for AndN {
    fn recv(&mut self) {
        // allは短絡評価なので注意
        // 正格評価にする
        self.cache = self.input.iter()
            .map(|input| input.recv().unwrap_or(false))
            .collect::<Vec<bool>>().into_iter()
            .all(|b| b);
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.output.send(self.cache)
    }
}
impl AndN {
    fn new(input: Vec<Receiver<bool>>, output: Sender<bool>) -> Self {
        Self {cache: false, input, output}
    }
    fn create(input: Vec<Receiver<bool>>) -> (Self, Receiver<bool>) {
        let (s, r) = mpsc::channel();
        (Self::new(input, s), r)
    }
}

#[derive(Debug)]
struct Or {
    cache: bool,
    input1: Receiver<bool>,
    input2: Receiver<bool>,
    output: Sender<bool>,
}

impl Component for Or {
    fn recv(&mut self) {
        let input1 = self.input1.recv().unwrap_or(false);
        let input2 = self.input2.recv().unwrap_or(false);
        self.cache = input1 || input2;
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.output.send(self.cache)
    }
}
impl Or {
    fn new(
        input1: Receiver<bool>,
        input2: Receiver<bool>,
        output: Sender<bool>,
    ) -> Self {
        Self {cache: false, input1, input2, output}
    }
    fn create(
        input1: Receiver<bool>,
        input2: Receiver<bool>,
    ) -> (Self, Receiver<bool>) {
        let (s, r) = mpsc::channel();
        (Self::new(input1, input2, s), r)
    }
}

#[derive(Debug)]
struct OrN {
    cache: bool,
    input: Vec<Receiver<bool>>,
    output: Sender<bool>,
}

impl Component for OrN {
    fn recv(&mut self) {
        self.cache = self.input.iter()
            .map(|r| r.recv().unwrap_or(false))
            .collect::<Vec<_>>()
            .iter()
            .any(|&b| b);
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.output.send(self.cache)
    }
}
impl OrN {
    fn new(
        input: Vec<Receiver<bool>>,
        output: Sender<bool>,
    ) -> Self {
        Self {cache: false, input, output}
    }
    fn create(
        input: Vec<Receiver<bool>>,
    ) -> (Self, Receiver<bool>) {
        let (s, r) = mpsc::channel();
        (Self::new(input, s), r)
    }
}

#[derive(Debug)]
struct Not {
    cache: bool,
    input: Receiver<bool>,
    output: Sender<bool>,
}

impl Component for Not {
    fn recv(&mut self) {
        let input = self.input.recv().unwrap_or(false);
        self.cache = !input;
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.output.send(self.cache)
    }
}

impl Not {
    fn new(
        input: Receiver<bool>,
        output: Sender<bool>,
    ) -> Self {
        Self {cache: false, input, output}
    }
    fn create(input: Receiver<bool>) -> (Self, Receiver<bool>) {
        let (s, r) = mpsc::channel();
        (Self::new(input, s), r)
    }
}

#[derive(Debug)]
struct Branch {
    cache: bool,
    input: Receiver<bool>,
    output1: Sender<bool>,
    output2: Sender<bool>,
}

impl Component for Branch {
    fn recv(&mut self) {
        let input = self.input.recv().unwrap_or(false);
        self.cache = input;
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.output1.send(self.cache)?;
        self.output2.send(self.cache)
    }
}

impl Branch {
    fn new(
        input: Receiver<bool>,
        output1: Sender<bool>,
        output2: Sender<bool>,
    ) -> Self {
        Self {cache: false, input, output1, output2}
    }
    fn create(input: Receiver<bool>) -> (Self, (Receiver<bool>, Receiver<bool>)) {
        let (output1_s, output1_r) = mpsc::channel();
        let (output2_s, output2_r) = mpsc::channel();
        (
            Self {cache: false, input, output1: output1_s, output2: output2_s},
            (output1_r, output2_r)
        )
    }
}

#[derive(Debug)]
struct BranchN {
    cache: bool,
    input: Receiver<bool>,
    output: Vec<Sender<bool>>,
}

impl Component for BranchN {
    fn recv(&mut self) {
        let input = self.input.recv().unwrap_or(false);
        self.cache = input;
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        for output in self.output.iter() {
            output.send(self.cache)?;
        }
        Ok(())
    }
}
impl BranchN {
    fn new(input: Receiver<bool>, output: Vec<Sender<bool>>) -> Self {
        Self {input, output, cache: false}
    }
    fn create(input_r: Receiver<bool>, n: u32) -> (Self, Vec<Receiver<bool>>) {
        let (output_s, output_r): (Vec<_>, Vec<_>) = (0..n).map(|_| mpsc::channel()).unzip();
        let branch = Self::new(input_r, output_s);
        (branch, output_r)
    }
}

struct NAND {
    and: And,
    not: Not,
}
impl Component for NAND {
    fn recv(&mut self) {
        self.and.recv();
        self.not.recv();
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.not.send()?;
        self.and.send()
    }
    fn needed_step(&self) -> usize {
        2
    }
}

impl NAND {
    fn new(input1: Receiver<bool>, input2: Receiver<bool>, output: Sender<bool>) -> Self {
        let (s, r) = mpsc::channel();
        let and = And::new(input1, input2, s);
        let not = Not::new(r, output);
        NAND {and, not}
    }
}

struct RSFlipFlop {
    nand1: NAND,
    nand2: NAND,
    not1: Not,
    not2: Not,
    branch1: Branch,
    branch2: Branch,
}

impl Component for RSFlipFlop {
    fn recv(&mut self) {
        self.nand1.recv();
        self.nand2.recv();
        self.not1.recv();
        self.not2.recv();
        self.branch1.recv();
        self.branch2.recv();
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.nand1.send()?;
        self.nand2.send()?;
        self.not1.send()?;
        self.not2.send()?;
        self.branch1.send()?;
        self.branch2.send()
    }
    fn needed_step(&self) -> usize {
        2 * self.nand1.needed_step()
            + 2 * self.not1.needed_step()
            + 2 * self.branch1.needed_step()
    }
}

impl RSFlipFlop {
    fn new(
        input_r: Receiver<bool>,
        input_s: Receiver<bool>,
        output_q: Sender<bool>,
        output_nq: Sender<bool>
    ) -> Self {
        let (n1_nand1_s, n1_nand1_r) = mpsc::channel();
        let (n2_nand2_s, n2_nand2_r) = mpsc::channel();
        let (nand1_branch1_s, nand1_branch1_r) = mpsc::channel();
        let (branch1_nand2_s, branch1_nand2_r) = mpsc::channel();
        let (nand2_branch2_s, nand2_branch2_r) = mpsc::channel();
        let (branch2_nand1_s, branch2_nand1_r) = mpsc::channel();
        let not1 = Not::new(input_s, n1_nand1_s);
        let not2 = Not::new(input_r, n2_nand2_s);
        let nand1 = NAND::new(n1_nand1_r, branch2_nand1_r, nand1_branch1_s);
        let nand2 = NAND::new(n2_nand2_r, branch1_nand2_r, nand2_branch2_s);
        let branch1 = Branch::new(nand1_branch1_r, output_q, branch1_nand2_s);
        let branch2 = Branch::new(nand2_branch2_r, output_nq, branch2_nand1_s);
        RSFlipFlop { nand1, nand2, not1, not2, branch1, branch2}
    }
    fn create(
        reset: Receiver<bool>,
        set: Receiver<bool>
    ) -> (Self, Receiver<bool>, Receiver<bool>) {
        let (q_s, q_r) = mpsc::channel();
        let (nq_s, nq_r) = mpsc::channel();
        let ff = Self::new(reset, set, q_s, nq_s);
        (ff, q_r, nq_r)
    }
    fn debug() -> Computer {
        let (s_s, s_r) = mpsc::channel();
        let (r_s, r_r) = mpsc::channel();
        let (q_s, q_r) = mpsc::channel();
        let (nq_s, nq_r) = mpsc::channel();
        let flipflop = Self::new(s_r, r_r, q_s, nq_s);
        let block = Block {comps: vec![Box::new(flipflop)]};
        let input = vec![s_s, r_s];
        let output = vec![q_r, nq_r];
        Computer {input, output, block, input_cache: vec![false, false]}
    }
}

struct BitDecoder {
    not: Vec<Not>,
    branch: Vec<BranchN>,
    and: Vec<AndN>,
}

impl Component for BitDecoder {
    fn recv(&mut self) {
        self.not.par_iter_mut().for_each(|not| {
            not.recv();
        });
        self.branch.par_iter_mut().for_each(|branch| {
            branch.recv();
        });
        self.and.par_iter_mut().for_each(|and| {
            and.recv();
        });
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.iter().into_iter().map(|c| c.send().map(|_| 0))
            .sum::<Result<i32, _>>()
            .map(|_| ())
    }
    fn needed_step(&self) -> usize {
        let margin = 2;
        2 * self.branch.get(0).map(|b| b.needed_step()).unwrap_or(1)
            + self.not.get(0).map(|n| n.needed_step()).unwrap_or(1)
            + self.and.get(0).map(|a| a.needed_step()).unwrap_or(1)
            + margin
    }
}

impl BitDecoder {
    fn create(input_r: Vec<Receiver<bool>>) -> (Self, Vec<Receiver<bool>>) {
        let bit = input_r.len() as u32;
        let masks: Vec<(usize, usize)> = (0..bit as usize).map(|i| (i, 1 << i)).collect();

        let (top_branch, top_input): (Vec<_>, Vec<Vec<_>>) = input_r.into_iter()
            .map(|input| BranchN::create(input, 2_u32.pow(bit - 1) + 1))
            .unzip();

        let (bot_input, top_input): (Vec<_>, Vec<Vec<_>>) = top_input.into_iter()
            .filter_map(|mut input| {
                let fst = input.pop();
                fst.zip(Some(input))
            })
            .unzip();
        let (not, bot_input): (Vec<Not>, Vec<_>) = bot_input.into_iter()
            .map(|input| Not::create(input))
            .unzip();
        let (bot_branch, bot_input): (Vec<BranchN>, Vec<Vec<_>>) = bot_input.into_iter()
            .map(|input| BranchN::create(input, 2_u32.pow(bit - 1)))
            .unzip();

        let mut top_input: Vec<_> = top_input.into_iter()
            .map(|input| input.into_iter())
            .collect();
        let mut bot_input: Vec<_> = bot_input.into_iter()
            .map(|input| input.into_iter())
            .collect();

        let (andn, output): (Vec<_>, Vec<_>) = (0..2_usize.pow(bit)).map(|i| {
            let input: Vec<_> = masks.iter()
                .map(|(j, mask)| match mask & i {
                    0 => bot_input[*j].next().unwrap(),
                    _ => top_input[*j].next().unwrap(),
                }).collect();
            AndN::create(input)
        }).unzip();

        let branch: Vec<BranchN> = top_branch.into_iter()
            .chain(bot_branch)
            .collect();
        (Self { not, branch, and: andn}, output)
    }

    fn debug() -> Computer {
        let bit = 8;
        let (input_s, input_r): (Vec<_>, _) = (0..bit).map(|_| mpsc::channel()).unzip();
        let (decoder, output) = BitDecoder::create(input_r);
        let block = Block {comps: vec![Box::new(decoder)]};
        let computer = Computer {input: input_s, output, block, input_cache: vec![false; bit]};
        return computer;
    }
    fn iter(&self) -> Vec<&dyn Component> {
        self.branch.iter().map(|b| b as &dyn Component)
            .chain(self.and.iter().map(|a| a as &dyn Component))
            .chain(self.not.iter().map(|n| n as &dyn Component))
            .collect()
    }
}

struct MemoryCell {
    ff: RSFlipFlop,
    branch: Vec<Branch>,
    nq: Receiver<bool>,
    not: Not,
    set: And,
    reset: And,
    read: And,
}

impl Component for MemoryCell {
    fn recv(&mut self) {
        self.iter_mut().par_iter_mut().for_each(|c| {
            c.recv();
        });
        self.nq.recv().unwrap_or(false);
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        for c in self.iter() {
            c.send()?;
        }
        Ok(())
    }
    fn needed_step(&self) -> usize {
        let margin = 2;
        self.branch.get(0).map(|b| b.needed_step()).unwrap_or(1)
            + self.not.needed_step()
            + self.reset.needed_step()
            + self.ff.needed_step()
            + self.read.needed_step()
            + margin
    }
}

impl MemoryCell {
    fn create(
        value: Receiver<bool>,
        read: Receiver<bool>,
        write: Receiver<bool>
    ) -> (Self, Receiver<bool>) {
        let (branch1, (b1_set_and, b1_reset_not)) = Branch::create(value);
        let (branch2, (b2_reset_and, b2_set_and)) = Branch::create(write);
        let (not, not_reset_and) = Not::create(b1_reset_not);
        let (and1, reset) = And::create(not_reset_and, b2_reset_and);
        let (and2, set) = And::create(b1_set_and, b2_set_and);
        let (ff, q, nq) = RSFlipFlop::create(reset, set);
        let (read_select, output) = And::create(read, q);
        let cell = MemoryCell {
            ff,
            branch: vec![branch1, branch2],
            nq,
            not,
            reset: and2,
            set: and1,
            read: read_select,
        };
        (cell, output)
    }
    fn debug() -> Computer {
        let (value_s, value_r) = mpsc::channel();
        let (read_s, read_r) = mpsc::channel();
        let (write_s, write_r) = mpsc::channel();
        let (cell, output) = Self::create(value_r, read_r, write_r);
        let block = Block {comps: vec![Box::new(cell)]};
        let computer = Computer {
            input: vec![value_s, read_s, write_s],
            output: vec![output],
            block,
            input_cache: vec![false; 3],
        };
        return computer;
    }
    fn iter(&self) -> Vec<Box<&dyn Component>> {
        self.branch.iter()
            .map(|b| Box::new(b as &dyn Component))
            .chain(vec![
                   Box::new(&self.not as &dyn Component),
                   Box::new(&self.set as &dyn Component),
                   Box::new(&self.reset as &dyn Component),
                   Box::new(&self.ff as &dyn Component),
                   Box::new(&self.read as &dyn Component),
            ])
            .collect()
    }
    fn iter_mut(&mut self) -> Vec<Box<&mut (dyn Component + Send)>> {
        self.branch.iter_mut()
            .map(|b| Box::new(b as &mut (dyn Component + Send)))
            .chain(vec![
                   Box::new(&mut self.not as &mut (dyn Component + Send)),
                   Box::new(&mut self.set as &mut (dyn Component + Send)),
                   Box::new(&mut self.reset as &mut (dyn Component + Send)),
                   Box::new(&mut self.ff as &mut (dyn Component + Send)),
                   Box::new(&mut self.read as &mut (dyn Component + Send)),
            ])
            .collect()
    }
}

struct MemoryByte {
    cells: Vec<MemoryCell>,
    write_branch: BranchN,
    read_branch: BranchN,
}

impl Component for MemoryByte {
    fn recv(&mut self) {
        self.write_branch.recv();
        self.read_branch.recv();
        self.cells.par_iter_mut().for_each(|cell| {
            cell.recv();
        });
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.write_branch.send()?;
        self.read_branch.send()?;
        for cell in self.cells.iter() {
            cell.send()?;
        }
        Ok(())
    }
    fn needed_step(&self) -> usize {
        let margin = 2;
        self.cells.get(0).map(|c| c.needed_step()).unwrap_or(1)
            + self.write_branch.needed_step()
            + self.read_branch.needed_step()
            + margin
    }
}

impl MemoryByte {
    fn create(
        values: Vec<Receiver<bool>>,
        write_select: Receiver<bool>,
        read_select: Receiver<bool>
    ) -> (Self, Vec<Receiver<bool>>) {
        let bit = values.len() as u32;
        let (write_branch, write_selects) = BranchN::create(write_select, bit);
        let (read_branch, read_selects) = BranchN::create(read_select, bit);
        let (cells, output): (Vec<_>, Vec<_>) = write_selects.into_iter()
            .zip(read_selects.into_iter())
            .zip(values.into_iter())
            .map(|((w, r), v)| MemoryCell::create(v, r, w))
            .unzip();
        let byte = Self {cells, read_branch, write_branch};
        (byte, output)
    }
    fn debug() -> Computer {
        let bit = 8;
        let (value_s, value_r) = (0..bit).map(|_| mpsc::channel()).unzip();
        let (write_s, write_r) = mpsc::channel();
        let (read_s, read_r) = mpsc::channel();
        let (byte, output) = Self::create(value_r, write_r, read_r);
        let block = Block { comps: vec![Box::new(byte)]};
        let input = vec![read_s, write_s].into_iter()
            .chain::<Vec<_>>(value_s).collect();
        // read, writeの+2
        let computer = Computer {input, output, block, input_cache: vec![false; bit + 2]};
        return computer;
    }
}

fn transpose<T>(mat: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut trans = vec![];
    let row = mat.get(0).map(|r| r.len()).unwrap_or(0);
    let mut mat: Vec<_> = mat.into_iter().map(|r| r.into_iter()).collect();
    for _ in 0..row {
        trans.push(mat.iter_mut().map(|r| r.next().unwrap()).collect());
    }
    return trans;
}

struct Memory {
    decoder: BitDecoder,
    read_write_branch: Vec<Branch>,
    read_branch: BranchN,
    write_branch: BranchN,
    data_branch: Vec<BranchN>,
    read_and: Vec<And>,
    write_and: Vec<And>,
    bytes: Vec<MemoryByte>,
    bit_collect: Vec<OrN>,
}

impl Component for Memory {
    fn recv(&mut self) {
        self.iter_mut().par_iter_mut().for_each(|comp| {
            comp.recv();
        });
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        for comp in self.iter() {
            comp.send()?;
        }
        Ok(())
    }
    fn fast_send(&mut self) -> Result<(), SendError<bool>> {
        self.iter_mut().par_iter_mut().map(|comp| comp.send().map(|_| 0))
            .sum::<Result<i32, _>>()
            .map(|_| ())
    }
    fn needed_step(&self) -> usize {
        let margin = 2;
        self.decoder.needed_step()
            + self.read_write_branch.get(0).map(|b| b.needed_step()).unwrap_or(1)
            + self.read_branch.needed_step()
            + self.write_branch.needed_step()
            + self.read_and.get(0).map(|a| a.needed_step()).unwrap_or(1)
            + self.write_and.get(0).map(|a| a.needed_step()).unwrap_or(1)
            + self.bytes.get(0).map(|b| b.needed_step()).unwrap_or(1)
            + self.bit_collect.get(0).map(|c| c.needed_step()).unwrap_or(1)
            + margin
    }
}

impl Memory {
    fn create(
        address: Vec<Receiver<bool>>,
        read: Receiver<bool>,
        write: Receiver<bool>,
        data: Vec<Receiver<bool>>,
    ) -> (Self, Vec<Receiver<bool>>) {
        let byte_num = 2_u32.pow(address.len() as u32);
        let _bit_num = data.len();
        let (decoder, address_select) = BitDecoder::create(address);
        let (read_write_branch, address_select): (Vec<_>, Vec<_>) = address_select.into_iter()
            .map(|address| Branch::create(address))
            .unzip();
        let (read_select, write_select): (Vec<_>, Vec<_>) = address_select.into_iter().unzip();
        let (read_branch, read) = BranchN::create(read, byte_num);
        let (write_branch, write) = BranchN::create(write, byte_num);
        let (read_and, read): (Vec<_>, Vec<_>) = read.into_iter()
            .zip(read_select)
            .map(|(r, select)| And::create(r, select))
            .unzip();
        let (write_and, write): (Vec<_>, Vec<_>) = write.into_iter()
            .zip(write_select)
            .map(|(w, select)| And::create(w, select))
            .unzip();
        let (data_branch, data): (Vec<_>, Vec<_>) = data.into_iter()
            .map(|val| BranchN::create(val, byte_num))
            .unzip();
        let data_t = transpose(data);
        let (bytes, read_byte): (Vec<_>, Vec<Vec<_>>) = read.into_iter()
            .zip(write)
            .zip(data_t)
            .map(|((r, w), d)| MemoryByte::create(d, w, r))
            .unzip();
        let read_byte_t = transpose(read_byte);
        let (bit_collect, output): (Vec<_>, Vec<_>) = read_byte_t.into_iter()
            .map(|r| OrN::create(r))
            .unzip();
        let memory = Memory {
            decoder,
            read_write_branch,
            read_branch,
            write_branch,
            data_branch,
            read_and,
            write_and,
            bytes,
            bit_collect,
        };
        return (memory, output);
    }
    fn debug() -> Computer {
        let address_num = 8;
        let bit_num = 8;
        let (address_s, address_r): (Vec<_>, Vec<_>) = (0..address_num)
            .map(|_| mpsc::channel())
            .unzip();
        let (read_s, read_r) = mpsc::channel();
        let (write_s, write_r) = mpsc::channel();
        let (data_s, data_r): (Vec<_>, Vec<_>) = (0..bit_num)
            .map(|_| mpsc::channel())
            .unzip();
        let (memory, output) = Self::create(address_r, read_r, write_r, data_r);
        let input: Vec<_> = vec![read_s, write_s].into_iter()
            .chain(address_s)
            .chain(data_s)
            .collect();
        let block = Block {
            comps: vec![Box::new(memory)],
        };
        let input_cache = input.iter().map(|_| false).collect();
        let computer = Computer {
            input,
            output,
            block,
            input_cache,
        };
        return computer;
    }
    fn iter(&self) -> Vec<&dyn Component> {
        let branch = self.read_write_branch.iter()
            .map(|c| c as &dyn Component);
        let branch_n = vec![&self.read_branch, &self.write_branch].into_iter()
            .chain(self.data_branch.iter())
            .map(|c| c as &dyn Component);
        let and  = self.read_and.iter()
            .chain(self.write_and.iter())
            .map(|c| c as &dyn Component);
        let bytes = self.bytes.iter()
            .map(|c| c as &dyn Component);
        let or = self.bit_collect.iter()
             .map(|c| c as &dyn Component);
        let comps = vec![&self.decoder as &dyn Component].into_iter()
            .chain(branch)
            .chain(branch_n)
            .chain(and)
            .chain(bytes)
            .chain(or)
            .collect();
        return comps;
    }
    fn iter_mut(&mut self) -> Vec<&mut (dyn Component + Send)> {
        let branch = self.read_write_branch.iter_mut()
            .map(|c| c as &mut (dyn Component + Send));
        let branch_n = vec![&mut self.read_branch, &mut self.write_branch].into_iter()
            .chain(self.data_branch.iter_mut())
            .map(|c| c as &mut (dyn Component + Send));
        let and  = self.read_and.iter_mut()
            .chain(self.write_and.iter_mut())
            .map(|c| c as &mut (dyn Component + Send));
        let bytes = self.bytes.iter_mut()
            .map(|c| c as &mut (dyn Component + Send));
        let or = self.bit_collect.iter_mut()
            .map(|c| c as &mut (dyn Component + Send));
        let comps = vec![&mut self.decoder as &mut (dyn Component + Send)].into_iter()
            .chain(branch)
            .chain(branch_n)
            .chain(and)
            .chain(bytes)
            .chain(or)
            .collect();
        return comps;
    }
}

struct Clock {
    freq: u32,
    count: u32,
    output: Sender<bool>,
    cache: bool,
}

impl Component for Clock {
    fn recv(&mut self) {
        if self.count == 0 {
            self.cache = !self.cache;
            self.count = self.freq;
        }
        self.count -= 1;
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        self.output.send(self.cache)
    }
}

struct Block {
    comps: Vec<Box<dyn Component>>,
}

impl Component for Block {
    fn recv(&mut self) {
        for comp in self.comps.iter_mut() {
            comp.recv();
        }
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        for comp in self.comps.iter() {
            comp.send()?;
        }
        Ok(())
    }
    fn fast_send(&mut self) -> Result<(), SendError<bool>> {
        for comp in self.comps.iter_mut() {
            comp.fast_send()?;
        }
        Ok(())
    }
    fn needed_step(&self) -> usize {
        self.comps.iter().map(|comp| comp.needed_step()).sum()
    }
}

impl Block {
    fn from_text(text: &str) -> (Block, Vec<Sender<bool>>, Vec<Receiver<bool>>) {
        let mut text = text.split_whitespace()
            .map(String::from)
            .collect();
        let mut comps = vec![];
        let mut input = BTreeMap::new();
        let mut output = vec![];
        while let Ok(r) = Self::parser(&mut text, &mut comps, &mut input) {
            output.push(r);
        }
        let input = input.into_values().collect();
        let block = Block {comps};
        return (block, input, output);
    }
    fn parser(
        text:  &mut Vec<String>,
        comps: &mut Vec<Box<dyn Component>>,
        input: &mut BTreeMap<String, Sender<bool>>,
    ) -> Result<Receiver<bool>, String>{
        if let Some(op) = text.pop() {
            match op.as_str() {
                "and" => {
                    let input1 = Self::parser(text, comps, input)?;
                    let input2 = Self::parser(text, comps, input)?;
                    let (s, r) = mpsc::channel();
                    let and = And::new(input1, input2, s);
                    comps.push(Box::new(and));
                    Ok(r)
                },
                "or" => {
                    let input1 = Self::parser(text, comps, input)?;
                    let input2 = Self::parser(text, comps, input)?;
                    let (s, r) = mpsc::channel();
                    let or = Or::new(input1, input2, s);
                    comps.push(Box::new(or));
                    Ok(r)
                },
                "not" => {
                    let input1 = Self::parser(text, comps, input)?;
                    let (s, r) = mpsc::channel();
                    let not = Not::new(input1, s);
                    comps.push(Box::new(not));
                    Ok(r)
                },
                op => {
                    let (s, r) = mpsc::channel();
                    if let Some(branch_s) = input.insert(op.to_string(), s) {
                        let (s2, r2) = mpsc::channel();
                        let branch = Branch::new(r, branch_s, s2);
                        comps.push(Box::new(branch));
                        Ok(r2)
                    } else {
                        Ok(r)
                    }
                }
            }
        } else {
            Err("too short".to_string())
        }
    }
}

struct Computer {
    input: Vec<Sender<bool>>,
    output: Vec<Receiver<bool>>,
    block: Block,
    input_cache: Vec<bool>,
}

impl Computer {
    fn from_text(text: &str) -> Self {
        let (block, input, output) = Block::from_text(text);
        let input_cache = vec![false; input.len()];
        Computer {input, output, block, input_cache}
    }
    fn step(&mut self, input: Vec<bool>) -> Result<Vec<bool>, SendError<bool>> {
        for (cache, f) in self.input_cache.iter_mut().zip(&input) {
            *cache = *f;
        }
        for (i, s) in self.input.iter().enumerate() {
            let f = input.get(i).cloned().unwrap_or(false);
            s.send(f)?;
        }
        self.block.fast_send()?;
        self.block.recv();
        Ok(self.output.iter()
            .map(|r| r.recv().unwrap_or(false))
            .collect())
    }
    fn step_with_cache(&mut self) -> Result<Vec<bool>, SendError<bool>> {
        self.step(self.input_cache.clone())
    }
    fn needed_step(&self) -> usize {
        self.block.needed_step()
    }
    fn init_circuit(&mut self, input: Vec<bool>) -> Result<Vec<bool>, SendError<bool>> {
        self.step(input)?;
        for _ in 0..self.needed_step() {
            self.step_with_cache()?;
        }
        self.step_with_cache()
    }
}

fn main() {
    // let mut c = Computer::from_text("i1 not i2 i2 and or");
    // println!("{:?}", c.init_circuit(vec![true, true, false]));
    // let mut flipflop = RSFlipFlop::debug();
    // println!("{:?}", flipflop.init_circuit(vec![true, false]));
    // println!("{:?}", flipflop.init_circuit(vec![false, false]));
    // println!("{:?}", flipflop.init_circuit(vec![false, true]));
    // println!("{:?}", flipflop.init_circuit(vec![false, false]));
    // println!("unstable");
    // println!("{:?}", flipflop.init_circuit(vec![true, true]));
    // println!("{:?}", flipflop.init_circuit(vec![false, false]));
    // for _ in 0..20 {
    //     println!("{:?}", flipflop.step_with_cache());
    // }
    // let mut decoder = BitDecoder::debug();
    // let bit = 8;
    // let masks: Vec<i32> = (0..bit).map(|i| 1 << i).collect();
    // for i in 0..2_i32.pow(bit) {
    //     let input: Vec<bool> = masks.iter().map(|mask| mask & i != 0).collect();
    //     decoder.init_circuit(input.clone()).unwrap();
    //     decoder.init_circuit(input.clone()).unwrap();
    //     println!("{:?}", decoder.init_circuit(input).unwrap().iter().position(|&p| p).unwrap());
    // }
    // let mut memory_cell = MemoryCell::debug();
    // let write = |cell: &mut Computer, d: bool| -> bool{
    //     cell.init_circuit(vec![d, true, true]).unwrap()[0]
    // };
    // write(&mut memory_cell, false);
    // println!("{:?}", memory_cell.init_circuit(vec![false, true, false]));
    // write(&mut memory_cell, true);
    // println!("{:?}", memory_cell.init_circuit(vec![false, true, false]));
    // for _ in 0..5 {
    //     println!("{:?}", write(&mut memory_cell, false));
    //     println!("{:?}", write(&mut memory_cell, true));
    // }
    // let mut memory_byte = MemoryByte::debug();
    // let bit = 8;
    // let read = |byte: &mut Computer| -> u32 {
    //     let mut flag = vec![false; bit + 2];
    //     flag[0] = true;
    //     let output = byte.init_circuit(flag).unwrap();
    //     output.into_iter().enumerate()
    //         .map(|(i, b)| (1 << i) * if b {1} else{0})
    //         .sum()
    // };
    // let write = |byte: &mut Computer, data: u32| -> u32 {
    //     let masks: Vec<u32> = (0..bit).map(|i| 1 << i).collect();
    //     let input: Vec<bool> = vec![false, true].into_iter()
    //         .chain(masks.iter().map(|mask| mask & data != 0))
    //         .collect();
    //     byte.init_circuit(input).unwrap();
    //     read(byte)
    // };
    // // 論理エラー
    // // 0~63の範囲しかできない
    // // encoding, decodingは正常
    // for i in 0..2_u32.pow(bit as u32) {
    //     write(&mut memory_byte, i);
    //     println!("{:?}", read(&mut memory_byte));
    // }
    let mut memory = Memory::debug();
    let address_bit = 8;
    let memory_bit = 8;
    let masks: Vec<_> = (0..memory_bit)
        .map(|i| 1 << i)
        .collect();
    let num_to_bit = |num: u32| -> Vec<bool> {
        masks.iter()
            .map(|mask| mask & num != 0)
            .collect()
    };
    let bit_to_num = |bits: Vec<bool>| -> u32 {
        bits.iter().enumerate()
            .map(|(i, &b)| (1 << i) * if b {1} else {0})
            .sum()
    };
    let read = |memory: &mut Computer, address: u32| -> u32 {
        let address_bit = num_to_bit(address);
        let flag = vec![true, false];
        let pad = vec![false; memory_bit];
        let input = flag.into_iter()
            .chain(address_bit)
            .chain(pad)
            .collect();
        let output = memory.init_circuit(input).unwrap();
        bit_to_num(output)
    };
    let write = |memory: &mut Computer, address: u32, data: u32| {
        let address_bit = num_to_bit(address);
        let data_bit = num_to_bit(data);
        let flag = vec![false, true];
        let input = flag.into_iter()
            .chain(address_bit)
            .chain(data_bit)
            .collect();
        memory.init_circuit(input).unwrap();
    };
    for i in 0..16 {
        write(&mut memory, i, i);
        println!("write {:?} in address: {:?}", i, i);
    }
    for i in 16..2_u32.pow(address_bit){
        write(&mut memory, i, 0);
        println!("write {:?} in address: {:?}", 0, i);
    }
    for i in 0..2_u32.pow(address_bit) {
        println!("{:?}", read(&mut memory, i));
    }
}
