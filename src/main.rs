use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, Sender, SendError, self};

trait Component {
    fn recv(&mut self);
    fn send(&self) -> Result<(), SendError<bool>>;
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
}

#[derive(Debug)]
struct AndN {
    cache: bool,
    input: Vec<Receiver<bool>>,
    output: Sender<bool>,
}
impl Component for AndN {
    fn recv(&mut self) {
        self.cache = self.input.iter()
            .all(|input| input.recv().unwrap_or(false));
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
        input_s: Receiver<bool>,
        input_r: Receiver<bool>,
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

struct EightBitDecoder {
    not: Vec<Not>,
    branch: Vec<BranchN>,
    and: Vec<AndN>,
}

impl Component for EightBitDecoder {
    fn recv(&mut self) {
        for not in self.not.iter_mut() {
            not.recv();
        }
        for branch in self.branch.iter_mut() {
            branch.recv();
        }
        for and in self.and.iter_mut() {
            and.recv();
        }
    }
    fn send(&self) -> Result<(), SendError<bool>> {
        for not in self.not.iter() {
            not.send()?;
        }
        for branch in self.branch.iter() {
            branch.send()?;
        }
        for and in self.and.iter() {
            and.send()?;
        }
        Ok(())
    }
    fn needed_step(&self) -> usize {
        self.not.iter().map(|n| n.needed_step()).sum::<usize>()
            + self.branch.iter().map(|b| b.needed_step()).sum::<usize>()
            + self.and.iter().map(|a| a.needed_step()).sum::<usize>()
    }
}

impl EightBitDecoder {
    fn create(input_r: Vec<Receiver<bool>>) -> (Self, Vec<Receiver<bool>>) {
        let bit = 8_u32;
        let masks: Vec<(usize, usize)> = (0..bit as usize).map(|i| (i, 1 << i)).collect();
        let (top_branch, top_input): (Vec<_>, Vec<Vec<_>>) = input_r.into_iter()
            .map(|input| BranchN::create(input, 2_u32.pow(bit - 1) + 1))
            .unzip();
        let (bot_input, top_input): (Vec<_>, Vec<Vec<_>>) = top_input.into_iter()
            .map(|mut input| {
                let fst = input.pop();
                fst.zip(Some(input))
            })
            .flatten()
            .unzip();
        let (not, bot_input): (Vec<_>, Vec<_>) = bot_input.into_iter()
            .map(|input| Not::create(input))
            .unzip();
        let (bot_branch, bot_input): (Vec<_>, Vec<Vec<_>>) = bot_input.into_iter()
            .map(|input| BranchN::create(input, 2_u32.pow(bit - 1)))
            .unzip();
        let mut top_input: Vec<_> = top_input.into_iter()
            .map(|input| input.into_iter())
            .collect();
        let mut bot_input: Vec<_> = bot_input.into_iter()
            .map(|input| input.into_iter())
            .collect();
        let (andn, output): (Vec<_>, Vec<_>) = (0..2_usize.pow(bit)).map(|i| {
            let input = masks.iter()
                .map(|(j, mask)| match mask & i {
                    // panic
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
        let (input_s, input_r): (Vec<_>, _) = (0..8).map(|_| mpsc::channel()).unzip();
        let (decoder, output) = EightBitDecoder::create(input_r);
        let block = Block {comps: vec![Box::new(decoder)]};
        let computer = Computer {input: input_s, output, block, input_cache: vec![false; 8]};
        return computer;
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
        self.block.send()?;
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
    let mut decoder = EightBitDecoder::debug();
    let bit = 8;
    let masks: Vec<i32> = (0..bit).map(|i| 1 << i).collect();
    // for i in 0..2_i32.pow(bit) {
    //     let input: Vec<bool> = masks.iter().map(|mask| mask & i != 0).collect();
    //     decoder.init_circuit(input.clone()).unwrap();
    //     decoder.init_circuit(input.clone()).unwrap();
    //     println!("{:0>8b}", decoder.init_circuit(input).unwrap().iter().position(|&p| p).unwrap());
    // }
    println!("{:?}", decoder.needed_step());
    // 論理エラー
    for _ in 0..10 {
        println!("{:0>8b}", decoder.init_circuit(vec![false; 8]).unwrap().iter().position(|&p| p).unwrap());
        println!("{:0>8b}", decoder.init_circuit(vec![true; 8]).unwrap().iter().position(|&p| p).unwrap());
    }
}
