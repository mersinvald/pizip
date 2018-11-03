use structopt::StructOpt;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "hex2bin")]
struct CliArgs {
    #[structopt(short = "v")]
    /// Enable verbose output
    verbose: bool,

    #[structopt(short = "i")]
    /// Input ASCII hex file path
    input: PathBuf,

    #[structopt(short = "o")]
    /// Output binary file path
    output: PathBuf
}

use log::{debug, info};
use memmap::MmapOptions;
use std::io::Write;
use std::fs::{File, OpenOptions};
use failure::Error;
use rayon::prelude::*;

const MEGABYTE: usize = 1024 * 1024;
const MAX_MEM_UTIL: usize = MEGABYTE * 1024 * 8;
const MAX_INPUT_PASS_SIZE: usize = MAX_MEM_UTIL * 2;

fn main() -> Result<(), Error> {
    let args = CliArgs::from_args();

    // Setup logger
    if args.verbose {
        std::env::set_var("RUST_LOG", "DEBUG");
    } else {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::init();

    // MMap input file
    let input_file = File::open(&args.input)?;
    let input = unsafe { MmapOptions::new().map(&input_file)? };

    // Calculate the output file length
    // 1 byte is represented as 2 ASCII symbols (2 bytes)
    let input_len = input.len();
    let output_len = input_len / 2;

    // Open output file
    let mut output_file = OpenOptions::new().read(true).write(true).create(true).open(&args.output)?;
    output_file.set_len(output_len as u64)?;

    let passes = (input_len as f64 / MAX_INPUT_PASS_SIZE as f64).ceil() as usize;
    let mut bytes_written = 0;
    let mut next_pass_start = 0;
    for i in 0..passes {
        // Calculate the start and finish offsets of the current pass
        let mut pass_start = next_pass_start;
        let mut pass_end = pass_start + MAX_INPUT_PASS_SIZE;

        // Trim the '3.' if it's there in the beginning
        if input[1] == '.' as u8 && i == 0 {
            pass_start = 2
        } else {
            pass_end += 2
        };

        // Converge end to the input length
        if pass_end > input.len() {
            pass_end = input.len();
        }

        info!("pass {}: {}..{}", i + 1, pass_start, pass_end);
        let pass_data = &input[pass_start..pass_end];
        next_pass_start = pass_end;
        info!("{} MB / {} MB read", as_megabytes(next_pass_start - 1), as_megabytes(input.len()));

        let threads_cnt = num_cpus::get_physical();
        let mut chunk_len = pass_data.len() / threads_cnt;
        if chunk_len % 2 != 0 {
            chunk_len += 1;
        }

        debug!("max chunk len: {}", chunk_len);
        info!("starting hex2bin conversion in {} threads", threads_cnt);

        let mut out_chunks = pass_data.par_chunks(chunk_len).enumerate()
            .map(|(no, chunk)| {
                debug!("thread chunk len: {}", chunk.len());
                // prefetch
                let out = chunk.chunks(2).map(|byte| {
                    let high = hex_to_4bit(byte[0]);
                    let low = hex_to_4bit(byte[1]);
                    (high << 4) | (low & 0xF)
                }).collect::<Vec<u8>>();
                (no, out)
            })
            .collect::<Vec<_>>();

        info!("hex2bin conversion finished, outputting to file");

        // Sort by reading order
        out_chunks.sort_by_key(|(no, _)| *no);

        for (_, out_chunk) in out_chunks {
            output_file.write(&out_chunk)?;
            bytes_written += out_chunk.len();
            output_file.flush()?;
            info!("{} MB / {} MB written (cached)", as_megabytes(bytes_written), as_megabytes(output_len));
            debug!("{} / {} bytes written (cached)", bytes_written, output_len);
        }
    }

    info!("flushing the output");
    output_file.flush()?;

    Ok(())
}

fn as_megabytes(bytes: usize) -> u64 {
    (bytes as f64 / MEGABYTE as f64).ceil() as u64
}

fn hex_to_4bit(hex: u8) -> u8 {
    let c = hex as char;
    if c >= '0' && c <= '9' {
        hex - '0' as u8
    } else if c >= 'A' && c <= 'F' {
        hex - 'A' as u8 + 10
    } else if c >= 'a' && c <= 'f' {
        hex - 'a' as u8 + 10
    } else {
        unreachable!()
    }
}
