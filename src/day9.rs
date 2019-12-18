use crate::day2::op::OpCode;
use crate::day2::Error;
use crate::day2::{read_comma_file, IntCodeMachine};
use crate::day7::build_machine as _build_machine;

use std::sync::mpsc::channel;

pub fn build_machine(mem: Vec<i64>) -> IntCodeMachine {
    let mut machine = _build_machine(mem);
    machine.reg_opcode(op::MoveRel::code(), op::MoveRel::new);
    machine.reg_param_mode(2, rel::load, rel::store);
    machine
}

pub fn run() -> Result<String, Error> {
    let mut data = read_comma_file("input/day9.txt")?;
    data.extend(&vec![0; 2048]);

    let part_1 = {
        let (o_tx, o_rx) = channel();

        let mut machine = build_machine(data.clone());
        machine.wire_output(o_tx);
        let i_tx = machine.wire_input();
        i_tx.send(1).unwrap();
        let _ = machine.run().unwrap();
        o_rx.recv()?
    };

    let part_2 = {
        let (o_tx, o_rx) = channel();

        let mut machine = build_machine(data.clone());
        machine.wire_output(o_tx);
        let i_tx = machine.wire_input();
        i_tx.send(2).unwrap();
        let _ = machine.run().unwrap();
        o_rx.recv()?
    };

    Ok(format!("{} | {}", part_1, part_2))
}

pub mod rel {
    use crate::day2::Error;

    pub fn store(ptr: isize, mem: &mut [i64], value: i64, rel_base: isize) -> Result<(), Error> {
        let rel_offset = *mem.get(ptr as usize).ok_or(Error::MemoryError(ptr))?;
        let iptr = rel_offset as isize + rel_base;
        *mem.get_mut(iptr as usize).ok_or(Error::MemoryError(iptr))? = value;
        Ok(())
    }

    pub fn load(ptr: isize, mem: &[i64], rel_base: isize) -> Result<i64, Error> {
        assert!(ptr > 0);
        let rel_offset = *mem.get(ptr as usize).ok_or(Error::MemoryError(ptr))? as isize;
        let iptr = rel_offset + rel_base;
        let value = *mem.get(iptr as usize).ok_or(Error::MemoryError(iptr))?;
        Ok(value)
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::day2::Error;

        #[test]
        fn rel_get() {
            let mem = vec![12, 1, 100];
            assert_eq!(load(1, &mem, 1), Ok(100));
        }

        #[test]
        fn rel_get_backwards() {
            let mem = vec![12, 1, 100];
            assert_eq!(load(1, &mem, -1), Ok(12));
        }

        #[test]
        fn rel_store() {
            let mut mem = [0, 1];
            let r = store(0, &mut mem, -1, 0);
            assert_eq!(r, Ok(()));
            assert_eq!(mem, [-1, 1]);

            let mut mem = [0, 1];
            let r = store(1, &mut mem, -1, 0);
            assert_eq!(r, Ok(()));
            assert_eq!(mem, [0, -1]);

            let mut mem = [0, 1];
            let r = store(0, &mut mem, -1, 1);
            assert_eq!(r, Ok(()));
            assert_eq!(mem, [0, -1]);

            let mut mem = [0, 1];
            let r = store(1, &mut mem, -1, 1);
            assert_eq!(r, Err(Error::MemoryError(2)));

            let mut mem = [0, 1];
            let r = store(1, &mut mem, -1, -1);
            assert_eq!(r, Ok(()));
            assert_eq!(mem, [-1, 1]);

        }
    }
}

pub mod op {
    use crate::day2::op::OpCode;
    use crate::day2::param::{decompose_param, ParamReg};
    use crate::day2::{Error, LoadPtr};

    use std::sync::mpsc::{Receiver, Sender};

    pub struct MoveRel(LoadPtr);
    impl OpCode for MoveRel {
        fn new(reg: &ParamReg, param: i64) -> Result<Box<dyn OpCode>, Error> {
            let ps = decompose_param(param, MoveRel::width() as usize);
            Ok(Box::new(MoveRel(reg.get(ps[0])?.load)) as Box<dyn OpCode>)
        }

        fn execute(
            &self,
            ip: isize,
            mem: &mut [i64],
            _: &Option<Receiver<i64>>,
            _: &mut Option<Sender<i64>>,
            rel_base: &mut isize,
        ) -> Result<isize, Error> {
            let adj = self.0(ip + 1, mem, *rel_base)?;
            //let adj = *mem.get((ip + 1) as usize).ok_or(Error::MemoryError(ip+1))?;
            let nrel_base = *rel_base + adj as isize;
            println!("MOVREL {} + {} = {}", rel_base, adj, nrel_base);
            *rel_base = nrel_base;
            Ok(MoveRel::width() as isize)
        }

        fn code() -> i64 {
            9
        }

        fn width() -> usize {
            2
        }
    }

    impl std::fmt::Debug for MoveRel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MOVREL")
        }
    }

    #[cfg(test)]
    mod test {
        use crate::day9::build_machine;
        use super::*;
        use crate::day5::immediate;

        #[test]
        fn moverel() {
            let mut mem = vec![109, 19];
            let mut rel_base = 2000;
            let op = MoveRel(immediate::load);
            assert!(op
                .execute(0, &mut mem, &mut None, &mut None, &mut rel_base)
                .is_ok());
            assert_eq!(rel_base, 2019);

            let mut mem = vec![109, 1];
            rel_base = 1;
            assert!(op
                .execute(0, &mut mem, &mut None, &mut None, &mut rel_base)
                .is_ok());
            assert_eq!(rel_base, 2);
        }
    }
}

#[cfg(test)]
mod day9_test {
    use super::*;

    use std::sync::mpsc::channel;

    #[test]
    fn quine() {
        let o_mem = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let mut mem = o_mem.clone();
        let l = mem.len();
        mem.extend(&vec![0; 1024 - l]);
        println!("len is {}", mem.len());

        let mut machine = build_machine(mem.clone());
        let (tx, rx) = channel();
        machine.wire_output(tx);
        let _ = machine.run().unwrap();
        let mut output = Vec::new();
        while let Ok(v) = rx.recv() {
            output.push(v);
        }
        assert_eq!(output, o_mem);
    }

    #[test]
    fn big_num() {
        let mem = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];
        let mut machine = build_machine(mem);
        let (tx, rx) = channel();
        machine.wire_output(tx);
        let _ = machine.run().unwrap();
        let s = format!("{}", rx.recv().unwrap());
        assert_eq!(16, s.len());
    }

    #[test]
    fn print_big() {
        let mem = vec![104, 1125899906842624, 99];
        let mut machine = build_machine(mem);
        let (tx, rx) = channel();
        machine.wire_output(tx);
        let _ = machine.run().unwrap();
        assert_eq!(1125899906842624, rx.recv().unwrap());
    }
}
