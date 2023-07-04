mod table_formatter;
use table_formatter::{TableFormatter, row};

// const PRIME: u32 = 0x5dc706bd;
const PRIME: u32 = 0xa88d411b;
// const PRIME_INV: u32 = PRIME.wrapping_pow(0x7fffffff);
const WINDOW: usize = 32;
const MUL_OUT: u32 = PRIME.wrapping_pow(WINDOW as u32);

fn polynomial_hash(input: &[u8]) -> u32 {
    input.iter().fold(0, |acc, &byte| acc.wrapping_mul(PRIME).wrapping_add(byte as u32))
}

fn rolling_hash(input: &[u8]) -> impl Iterator<Item=u32> + '_ {
    // simple polynomial rolling hash
    input.iter().enumerate().scan(0u32, |acc, (i, &byte)| {
        let outgoing = if i < WINDOW { 0 } else { input[i - WINDOW] };
        *acc = (*acc)
            .wrapping_mul(PRIME)
            .wrapping_sub((outgoing as u32).wrapping_mul(MUL_OUT))
            .wrapping_add(byte as u32);
        Some(*acc)
    })
}

const CHUNK_AVERAGE_SIZE: usize = 64;

fn chunks(input: &[u8]) -> Vec<(u32, &[u8])> {
    const THRESHOLD: u32 = ((u64::MAX / CHUNK_AVERAGE_SIZE as u64) >> 32) as _;
    let mut chunks = Vec::new();
    let mut chunk_start = 0;
    for (i, h) in rolling_hash(input).enumerate() {
        let i = i + 1;
        if h <= THRESHOLD || i == input.len() {
            chunks.push((
                h,
                &input[chunk_start..i]
            ));
            chunk_start = i;
        }
    }
    chunks
}

fn main() -> std::io::Result<()> {
    use std::io::{BufReader, BufRead};
    use std::fs::File;
    use std::collections::HashSet;

    let file = BufReader::new(File::open("words.txt")?);
    let mut groups = Vec::new();
    for line in file.lines() {
        groups.push((false, line?));
    }

    let mut order = (0..groups.len()).collect::<Vec<_>>();
    order.sort_by_cached_key(|&i| polynomial_hash(groups[i as usize].1.as_bytes()));

    let mut chunks_unique = HashSet::<Vec<u8>>::new();
    let mut chunk_hashes = HashSet::<u32>::new();
    let mut chunk_refs = 0usize;
    let mut total_bytes = 0usize;
    let mut chunks_bytes = 0;

    print!("\
    text: size of the current version
   bytes: size of all versions w/ deduping (chunk data + chunk references)
   cumul: size of all versions
    uniq: # of distinct chunks
collides: # of hash collisions
");
    let mut formatter = TableFormatter::new([
        ("#", 4),
        ("text", 8),
        ("bytes", 8),
        ("cumul", 8),
        ("uniq", 8),
        ("collides", 10),
    ]);
    formatter.print_header();

    for (i, idx) in order.into_iter().enumerate() {
        let i = i + 1;

        // mark a new random group as active
        groups[idx].0 = true;

        // create a text out of all active groups
        let mut text = String::new();
        for (active, line) in &groups {
            if !active {
                continue;
            }
            text.push_str(line);
            text.push('\n');
        }

        // try to deduplicate this text
        let new_chunks = chunks(text.as_bytes());
        for &(hash, chunk) in &new_chunks {
            chunk_refs += 1;
            total_bytes += chunk.len();

            let new_chunk = !chunks_unique.contains(chunk);
            let new_hash = chunk_hashes.insert(hash);

            if false {
                if new_hash {
                    println!("new hash: {hash:0>8x}");
                } else if new_chunk {
                    println!("dup hash: {hash:0>8x}");
                }
            }

            if new_chunk {
                chunks_bytes += chunk.len();
                chunks_unique.insert(chunk.to_vec());
            }
        }

        let bytes_estimate = 4 * chunk_refs + chunks_bytes;

        row!(formatter, i, text.len(), bytes_estimate, total_bytes, chunks_unique.len(), chunks_unique.len() - chunk_hashes.len());
    }
    Ok(())
}
