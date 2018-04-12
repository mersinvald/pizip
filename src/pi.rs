use rayon::prelude::*;
use num::Float;
use std::mem;
use std::fmt;
use std::io::{stdout, Write};

pub fn pi_byte_sequence(ofst: i64, mut array: &mut [u8]) -> &mut [u8] {
    let size = array.len();
    array.par_iter_mut()
         .enumerate()
         .for_each(|(i, x)| {
            *x = pi_byte(ofst + i as i64);
         });
    array
}

pub fn pi_digit_sequence(ofst: i64, mut array: &mut [u8]) -> &mut [u8] {
    array.par_iter_mut()
         .enumerate()
         .for_each(|(i, x)| {
            *x = pi_digit(ofst + i as i64);
         });
    array
}

pub fn pi_byte(n: i64) -> u8 {
    let n = n * 2;
    let du = pi_digit(n) << 4;
    let dd = pi_digit(n+1);
    du | dd
}

pub fn pi_digit(n: i64) -> u8 {
    let n = (n - 1) as i64;
    let x = 4.0 * pi_term(1, n) 
          - 2.0 * pi_term(4, n) 
          - pi_term(5, n) 
          - pi_term(6, n);
    let x = x - x.floor();
    (x * 16.0) as u8
}

fn pi_term(j: i64, n: i64) -> f32 {
    // Calculate the left sum
    let mut s = 0.0;
    for k in 0..=n {
        let r = 8 * k + j;
        s += power_mod(16, n-k, r) as f32 / r as f32;
        s -= s.floor();
    }

    // Calculate the right sum
    let k = n + 1;
    let r = 8 * k + j;
    let t = 16.0.powi((n-k) as i32) / r as f32;

    let term = (s + t) - (s + t).floor();
    term
}


fn power_mod(a: i64, b: i64, m: i64) -> i64 {
    if b == 0 {
        1
    } else if b == 1 {
        a
    } else {
        let temp = power_mod(a, b / 2, m);
        if b % 2 == 0 {
            (temp * temp) % m
        } else {
            ((temp * temp) % m) * a % m
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn pi_digit() {
        assert_eq!(0x3, super::pi_digit(0));
        assert_eq!(0xF, super::pi_digit(4));
    }

    #[test]
    fn pi_sequence() {
        let right_sequence = [
            0x3,0x2,0x4,0x3,0xF,0x6, 
            0xA,0x8,0x8,0x8,0x5,
            0xA,0x3,0x0,0x8,0xD, 
            0x3,0x1,0x3,0x1,0x9,
            0x8,0xA,0x2,0xE,0x0,
            0x3,0x7,0x0,0x7,0x3, 
            0x4,0x4,0xA,0x4,0x0, 
            0x9,0x3,0x8,0x2,0x2, 
        ];

        let mut generated_sequence = vec![0; right_sequence.len()];

        assert_eq!(&right_sequence[..], super::pi_digit_sequence(0, &mut generated_sequence));
    }
}
