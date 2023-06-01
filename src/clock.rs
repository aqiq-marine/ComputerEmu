use crate::core::*;

pub struct Clock {
    s: bool,
}
impl Clock {
    pub fn new() -> Self {
        Self {s: false}
    }
}

impl Component<0, 1> for Clock {
    fn eval(&self, input: [bool; 0]) -> [bool; 1] {
        return [false];
    }
    fn eval_mut(&mut self, input: [bool; 0]) -> [bool; 1] {
        self.s = !self.s;
        return [self.s];
    }
}

#[test]
fn clock_test() {
    let mut clock = Clock::new();
    let mut s = false;
    assert_eq!(clock.s, s);

    for _ in 0..10 {
        s = !s;
        println!("clock: {}", clock.s);
        assert_eq!(clock.eval_mut([]), [s]);
    }
}

// クロックの立ち上がりの時Trueを返す
pub struct DetectClockWake {
    s: bool,
}
impl DetectClockWake {
    pub fn new() -> Self {
        Self {s: true}
    }
}

impl Component<1, 1> for DetectClockWake {
    fn eval(&self, input: [bool; 1]) -> [bool; 1] {
        return [false];
    }
    fn eval_mut(&mut self, input: [bool; 1]) -> [bool; 1] {
        let output = !self.s && input[0];
        self.s = input[0];
        return [output];
    }
}

#[test]
fn detect_clock_wake_test() {
    let mut clock_wake = DetectClockWake::new();

    assert_eq!([false], clock_wake.eval_mut([true]));
    assert_eq!([false], clock_wake.eval_mut([false]));
    assert_eq!([true], clock_wake.eval_mut([true]));
    assert_eq!([false], clock_wake.eval_mut([true]));
    assert_eq!([false], clock_wake.eval_mut([false]));

}
