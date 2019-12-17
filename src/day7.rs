use crate::day2::op::OpCode;
use crate::day2::*;
use crate::day5;
use crate::day5::immediate;
use op::*;
use std::sync::mpsc::*;

use permute;

pub fn run() -> Result<String, Error> {
    let data = read_comma_file("input/day7.txt")?;

    let result = permute::permutations_of(&[0, 1, 2, 3, 4])
        .map(|seq| {
            let mut cluster = Cluster::build(5, &data.clone());
            cluster.start().unwrap();
            let seq: Vec<i32> = seq
                .enumerate()
                .map(|(i, v)| {
                    cluster.input(i, *v).unwrap();
                    *v
                })
                .collect();

            // then input value
            cluster.input(0, 0).unwrap();

            cluster.finish().unwrap();
            (seq, cluster.read_output().unwrap())
        })
        .max_by_key(|r| r.1)
        .unwrap();

    let result_2 = permute::permutations_of(&[9,8,7,6,5])
        .map(|seq| {
            let mut cluster = Cluster::build(5, &data);
            cluster.start().unwrap();
            let seq: Vec<i32> = seq.enumerate().map(|(i, v)| {
                cluster.input(i, *v).unwrap();
                *v
            })
            .collect();

            let mut next_input = 0;
            while let Ok(_) = cluster.input(0, next_input) {
                next_input = cluster.read_output().unwrap();
            }
            cluster.finish().unwrap();
            (seq, next_input)
        })
        .max_by_key(|r| r.1)
        .unwrap();


    Ok(format!("{:?}, {} | {:?} {}", result.0, result.1, result_2.0, result_2.1))
}

pub struct Cluster {
    /// the IntCode machines, each with its output wired to the input of the next
    machines: Option<Vec<IntCodeMachine>>,
    /// the final output
    output: Receiver<i32>,
    /// an input for each machine
    inputs: Vec<Sender<i32>>,
    /// thread handles
    t_handles: Option<Vec<std::thread::JoinHandle<Result<Vec<i32>, Error>>>>,
}

impl Cluster {
    pub fn input(&mut self, id: usize, value: i32) -> Result<(), Error> {
        Ok(self
            .inputs
            .get(id)
            .ok_or(Error::UserInputFailed)?
            .send(value)?)
    }

    pub fn read_output(&mut self) -> Option<i32> {
        self.output
            .recv_timeout(std::time::Duration::from_secs(1))
            .ok()
    }

    pub fn build(num: usize, mem: &[i32]) -> Self {
        let mut machines = vec![build_module(mem.to_vec())];
        let mut terminals = Vec::new();

        for i in 1..num {
            let mut next = build_module(mem.to_vec());
            let tx = next.wire_input();
            machines[i - 1].wire_output(tx.clone());
            machines.push(next);
            terminals.push(tx);
        }
        let first_tx = machines[0].wire_input();
        terminals.insert(0, first_tx);
        let (last_tx, last_rx) = channel();
        machines[num - 1].wire_output(last_tx);

        Self {
            machines: Some(machines),
            output: last_rx,
            inputs: terminals,
            t_handles: None,
        }
    }

    pub fn start(&mut self) -> Result<(), ()> {
        if let Some(m) = self.machines.take() {
            self.t_handles = Some(
                m.into_iter()
                    .map(|m| std::thread::spawn(|| m.run()))
                    .collect(),
            );
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn finish(&mut self) -> Result<Vec<Vec<i32>>, Error> {
        if let Some(handles) = self.t_handles.take() {
            handles
                .into_iter()
                .map(move |h| h.join().unwrap())
                .collect()
        } else {
            Err(Error::NotRunning)
        }
    }
}

pub(crate) fn build_module(mem: Vec<i32>) -> IntCodeMachine {
    let mut m = IntCodeMachine::boot(mem);
    m.reg_opcode(WiredOutput::code(), WiredOutput::new);
    m.reg_opcode(WiredInput::code(), WiredInput::new);
    m.reg_param_mode(1, immediate::load, immediate::store);
    m.reg_opcode(day5::op::Jnz::code(), day5::op::Jnz::new);
    m.reg_opcode(day5::op::Jz::code(), day5::op::Jz::new);
    m.reg_opcode(day5::op::Eq::code(), day5::op::Eq::new);
    m.reg_opcode(day5::op::Lt::code(), day5::op::Lt::new);
    m
}

pub mod op {
    use crate::day2::indirect;
    use crate::day2::op::OpCode;
    use crate::day2::param::ParamReg;
    use crate::day2::Error;

    use std::sync::mpsc::{Receiver, Sender};

    pub struct WiredInput;
    impl OpCode for WiredInput {
        fn new(_reg: &ParamReg, _param: i32) -> Result<Box<dyn OpCode>, Error> {
            Ok(Box::new(WiredInput))
        }

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i32],
            inp: &Option<Receiver<i32>>,
            _: &mut Option<Sender<i32>>,
        ) -> Result<isize, Error> {
            let value = inp.as_ref().ok_or(Error::InputFailed)?.recv()?;
            indirect::store(ip + 1, mem, value)?;
            Ok(WiredInput::width() as isize)
        }

        fn width() -> usize {
            2
        }

        fn code() -> i32 {
            3
        }
    }

    impl std::fmt::Debug for WiredInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "IN")
        }
    }

    pub struct WiredOutput;
    impl OpCode for WiredOutput {
        fn new(_reg: &ParamReg, _param: i32) -> Result<Box<dyn OpCode>, Error> {
            Ok(Box::new(WiredOutput))
        }

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i32],
            _: &Option<Receiver<i32>>,
            out: &mut Option<Sender<i32>>,
        ) -> Result<isize, Error> {
            out.as_ref()
                .ok_or(Error::OutputFailed)?
                .send(indirect::load(ip + 1, mem)?)?;
            Ok(WiredOutput::width() as isize)
        }

        fn width() -> usize {
            2
        }

        fn code() -> i32 {
            4
        }
    }

    impl std::fmt::Debug for WiredOutput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "OUT")
        }
    }
}

#[cfg(test)]
mod test {
    use super::op::*;
    use super::*;
    use crate::day2::op::OpCode;
    use std::thread;

    #[test]
    fn wired_in() {
        let val = 123;
        let mem = vec![3, 1, 99];
        let mut m = IntCodeMachine::boot(mem);
        m.reg_opcode(WiredInput::code(), WiredInput::new);
        let tx = m.wire_input();
        let th = thread::spawn(move || m.run());
        tx.send(val).unwrap();
        let end_state = th.join().unwrap();
        assert_eq!(end_state, Ok(vec![3, val, 99]));
    }

    #[test]
    fn wired_out() {
        let val = 321;
        let mem = vec![4, 3, 99, val];
        let (tx, rx) = channel();
        let mut m = IntCodeMachine::boot(mem);
        m.reg_opcode(WiredOutput::code(), WiredOutput::new);
        m.wire_output(tx);
        let th = thread::spawn(move || m.run());
        let value = rx.recv().unwrap();
        let _ = th.join().unwrap();
        assert_eq!(value, val);
    }

    #[test]
    fn test_cluster() {
        let mem = vec![3, 11, 3, 12, 1, 11, 12, 11, 4, 11, 99, -1, -2];
        let mut cluster = Cluster::build(5, &mem);
        cluster.start().unwrap();
        cluster.input(0, 1).unwrap();
        cluster.input(0, 1).unwrap(); // 2
        cluster.input(1, 1).unwrap(); // 3
        cluster.input(2, 1).unwrap(); // 4
        cluster.input(3, 1).unwrap(); // 5
        cluster.input(4, 1).unwrap(); // 6
        let _results = cluster.finish();
        let fin = cluster.read_output().unwrap();
        assert_eq!(fin, 6);
    }

    #[test]
    fn test_cluster_ex() {
        let mem = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];
        let mut cluster = Cluster::build(5, &mem);
        cluster.start().unwrap();
        // input gain
        cluster.input(0, 4).unwrap();
        cluster.input(1, 3).unwrap();
        cluster.input(2, 2).unwrap();
        cluster.input(3, 1).unwrap();
        cluster.input(4, 0).unwrap();

        // then input value
        cluster.input(0, 0).unwrap();

        cluster.finish().unwrap();
        let fin = cluster.read_output().unwrap();
        assert_eq!(fin, 43210);

        let mem = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];
        let mut cluster = Cluster::build(5, &mem);
        cluster.start().unwrap();
        // input gain
        cluster.input(0, 0).unwrap();
        cluster.input(1, 1).unwrap();
        cluster.input(2, 2).unwrap();
        cluster.input(3, 3).unwrap();
        cluster.input(4, 4).unwrap();

        // then input value
        cluster.input(0, 0).unwrap();

        cluster.finish().unwrap();
        let fin = cluster.read_output().unwrap();
        assert_eq!(fin, 54321);

        let mem = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];
        let mut cluster = Cluster::build(5, &mem);
        cluster.start().unwrap();
        // input gain
        cluster.input(0, 1).unwrap();
        cluster.input(1, 0).unwrap();
        cluster.input(2, 4).unwrap();
        cluster.input(3, 3).unwrap();
        cluster.input(4, 2).unwrap();

        // then input value
        cluster.input(0, 0).unwrap();

        cluster.finish().unwrap();
        let fin = cluster.read_output().unwrap();
        assert_eq!(fin, 65210);
    }

    #[test]
    fn test_fb() {
        let data = vec![
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ];

        let mut cluster = Cluster::build(5, &data);
        cluster.start().unwrap();
        cluster.input(0, 9).unwrap();
        cluster.input(1, 8).unwrap();
        cluster.input(2, 7).unwrap();
        cluster.input(3, 6).unwrap();
        cluster.input(4, 5).unwrap();

        let mut next_input = 0;
        while let Ok(_) = cluster.input(0, next_input) {
            next_input = cluster.read_output().unwrap();
        }
        cluster.finish().unwrap();

        assert_eq!(139629729, next_input);
    }

    #[test]
    fn test_fb_2() {
        let data = vec![3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,
-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,
53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10];

        let mut cluster = Cluster::build(5, &data);
        cluster.start().unwrap();
        cluster.input(0, 9).unwrap();
        cluster.input(1, 7).unwrap();
        cluster.input(2, 8).unwrap();
        cluster.input(3, 5).unwrap();
        cluster.input(4, 6).unwrap();

        let mut next_input = 0;
        while let Ok(_) = cluster.input(0, next_input) {
            next_input = cluster.read_output().unwrap();
        }
        cluster.finish().unwrap();

        assert_eq!(18216, next_input);
    }
}
