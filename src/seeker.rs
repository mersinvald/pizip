use rayon::prelude::*;
use num::Float;
use std::mem;
use std::fmt;
use ::pi::*;

use std::fs::File;
use std::io;
use std::io::{Read, Write};

pub const PI_BLK_SIZE: usize = 2;
pub const PI_SPACE: usize = 0xFFFF;

pub struct PiSeeker {
    pi: Vec<u8>,
}

impl PiSeeker {
    pub fn precalculate() -> PiSeeker {
        let pi = if let Ok(Some(pi_cache)) = Self::try_load_cache() {
            pi_cache
        } else {
            let mut pi = Box::new([0; PI_SPACE]);
            let pi = pi_byte_sequence(0, &mut pi[..]);
            Self::save_cache(&pi).unwrap();
            Vec::from(pi)
        };
        
        PiSeeker {
            pi: pi
        }
    }

    fn try_load_cache() -> io::Result<Option<Vec<u8>>> {
        let mut cache_file = File::open("pi_cache")?;
        let mut buffer = Vec::with_capacity(PI_SPACE);
        cache_file.read_to_end(&mut buffer)?;
        if buffer.len() == PI_SPACE {
            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }

    fn save_cache(buffer: &[u8]) -> io::Result<()> {
        let mut cache_file = File::create("pi_cache")?;
        cache_file.write(buffer)?;
        Ok(())
    }

    pub fn seek(&self, buffer: &[u8]) -> Vec<PiBlock> {
        buffer.chunks(PI_BLK_SIZE)
              .map(|chunk| {
                  println!("{:?}", chunk);
                  for i in 0..(self.pi.len() - PI_BLK_SIZE) {
                      let si = i;
                      let ei = i + PI_BLK_SIZE;
                      if chunk.eq(&self.pi[si..ei]) {
                          println!("FOUND {:?} == {:?}", chunk, &self.pi[si..ei]);
                          return PiBlock::calc_seq(si);
                      } else {
                          println!("{:?} != {:?}", chunk, &self.pi[si..ei]);
                      }
                  }
                  panic!("Block search exceeded PiSpace({})", PI_SPACE);
              })
              .collect()
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct PiBlock {
    number: usize,
    raw: [u8; PI_BLK_SIZE]
}

impl PiBlock {
    pub fn calc(number: usize) -> Self {
        let mut raw = [0; PI_BLK_SIZE];
        pi_byte_sequence((number * PI_BLK_SIZE) as i64, &mut raw);
        PiBlock {
            number: number,
            raw: raw
        }
    }

    pub fn calc_seq(pi_ofst: usize) -> Self {
        let mut raw = [0; PI_BLK_SIZE];
        pi_byte_sequence((pi_ofst) as i64, &mut raw);
        PiBlock {
            number: pi_ofst,
            raw: raw
        }
    }
}

impl fmt::Display for PiBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PiBlock {{ number: {}, raw: [{:X}", self.number, self.raw[0])?;
        for digit in self.raw.iter().skip(1) {
            write!(f, ", {:X}", digit)?;
        }
        write!(f, "] }}")
    }
}

pub fn pi_blocks(ofst: usize, mut array: &mut [PiBlock]) -> &mut [PiBlock] {
    array.par_iter_mut()
         .enumerate()
         .for_each(|(i, x)| {
            *x = PiBlock::calc(ofst + i)
         });
    array
}

pub fn pi_blocks_seq(ofst: usize, mut array: &mut [PiBlock]) -> &mut [PiBlock] {
    array.par_iter_mut()
         .enumerate()
         .for_each(|(i, x)| {
            *x = PiBlock::calc_seq(ofst + i)
         });
    array
}
