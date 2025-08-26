#![allow(unused_parens, unused, non_snake_case)]
mod inputs;
use anyhow::{anyhow, bail, Result};
use inputs::*;
use rayon::prelude::*;

pub fn part1() -> Result<()>
{
    let mut pc = load_input();
    let mut r = pc.cycle();
    while r.is_ok()
    {
        r = pc.cycle();
        if pc.clock > 1000
        {
            bail!("Clock > 1000");
        }
    }
    println!("Output : {:?}", pc.output);
    println!("Exit Status : {:?}", r);
    return Ok(());
}

pub fn part2() -> Result<()>
{
    brute_force()
    // maybe if we know the output, we can reverse it to get the input
    // i think i need to make an inverse computer that takes the output and traverses through
    // the possible inputs at each instruction in a DFS manner but... seems complicated, dont have time.
}

pub fn brute_force() -> Result<()>
{
    let pc = load_input();

    (0..1_000_000_000).into_par_iter().for_each(|i| {
                                          let mut trial = pc.clone();
                                          trial.rA = i;
                                          while let Ok(_) = trial.cycle()
                                          {
                                              if pc.clock > 2500
                                              {
                                                  break;
                                              }
                                          }
                                          if (trial.output.starts_with(&trial.program))
                                          {
                                              println!("Solution found : {}", i);
                                          }
                                      });
    println!("Solution not found.");
    return Ok(());
}

/*
PROGRAM FORMAT
3 bit instruction, 3 bit operand, repeat.
the instruction pointer increments by 2, and the
computer reads (instruction, operand) each cycle.

INSTRUCTIONS
adv         combo   perform division. numerator is A register, denom is 2^(operand)
bxl         literal bitwise xor of register B and the oeprand then stores into B
bst         combo   calculate operand mod 8 then stores into B
jnz         literal nothing if reg. A is 0, otherwise jump to instruction (operand). no inc.
bxc                 calc bitwise XOR of reg B and reg C then stores into B.
out         combo   output the operand value. multiple outputs are comma separated.
bdv         combo   perform division like adv, but store into B
cdv         combo   also does division but store into C register

OPERANDS
literal operands represent their value.

combo operands are as follows :
0..3        the values 0..3
4.6         register a..c
7           invalid
*/

#[derive(Debug, Clone)]
pub struct Computer
{
    rA:          i64,
    rB:          i64,
    rC:          i64,
    instruction: usize,
    program:     Vec<u8>,
    clock:       u64,
    output:      Vec<u8>,
}

impl Computer
{
    pub fn cycle(&mut self) -> Result<()>
    {
        let icode = self.program
                        .get(self.instruction)
                        .ok_or(anyhow!("Halt at instruction {}. Clock : {}, \n{:?}",
                                       self.instruction,
                                       self.clock,
                                       self))?
                        .clone();
        let opcode = self.program
                         .get(self.instruction + 1)
                         .ok_or(anyhow!("Halt at instruction {}. Clock : {}, \n{:?}",
                                        self.instruction + 1,
                                        self.clock,
                                        self))?
                         .clone();
        let operand = self.load_operand(icode, opcode)?;

        match icode
        {
            0 => self.adv(operand),
            1 => self.bxl(operand),
            2 => self.bst(operand),
            3 => self.jnz(operand),
            4 => self.bxc(operand),
            5 => self.out(operand),
            6 => self.bdv(operand),
            7 => self.cdv(operand),
            _ => bail!("Invalid opcode : {}\n{:?}", icode, self),
        };

        if (icode != 3)
        {
            self.instruction += 2;
        }
        else if (self.rA == 0)
        {
            bail!("Halted");
        }
        self.clock += 1;
        return Ok(());
    }

    fn load_operand(&self, icode: u8, opcode: u8) -> Result<i64>
    {
        let res = match icode
        {
            0 | 2 | 5 | 6 | 7 => match opcode
            {
                0..=3 => opcode as i64,
                4 => self.rA,
                5 => self.rB,
                6 => self.rC,
                7 => bail!("Invalid combo operator : 7\n{:?}", self),
                x => bail!("Invalid combo operator : {}\n{:?}", x, self),
            },
            1 | 3 | 4 => opcode as i64,
            _ => bail!("Invalid icode : {}\n{:?}", icode, self),
        };
        return Ok(res);
    }

    pub fn adv(&mut self, operand: i64)
    {
        self.rA = self.rA / 2i64.pow(operand as u32);
    }

    pub fn bxl(&mut self, operand: i64)
    {
        self.rB = self.rB ^ operand;
    }

    pub fn bst(&mut self, operand: i64)
    {
        self.rB = operand % 8;
    }

    pub fn jnz(&mut self, operand: i64)
    {
        if (self.rA == 0)
        {
            return;
        }
        self.instruction = operand as usize;
    }

    pub fn bxc(&mut self, operand: i64)
    {
        self.rB = self.rB ^ self.rC;
    }

    pub fn out(&mut self, operand: i64)
    {
        //print!("{},", operand%8);
        self.output.push(operand as u8 % 8);
    }

    pub fn bdv(&mut self, operand: i64)
    {
        self.rB = self.rA / 2i64.pow(operand as u32);
    }

    pub fn cdv(&mut self, operand: i64)
    {
        self.rC = self.rA / 2i64.pow(operand as u32);
    }
}
