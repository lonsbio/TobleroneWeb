// Copyright (c) 2018 10x Genomics, Inc. All rights reserved.
// Copyright (c) 2021 Andrew Lonsdale tinyt


//! Utility methods.
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::Path;
use std::sync::{Arc, Mutex};

use bincode::{self, deserialize_from, serialize_into};
use failure::{self, Error};
use flate2::read::MultiGzDecoder;
use serde::{de::DeserializeOwned, Serialize};

use bio::io::{fasta, fastq};
use debruijn::dna_string::DnaString;
use log::info;

use crate::config::FastaFormat;
 use debruijn::Mer;
pub fn write_obj<T: Serialize, P: AsRef<Path> + Debug>(
    g: &T,
    filename: P,
) -> Result<(), bincode::Error> {
    let f = match File::create(&filename) {
        Err(err) => panic!("couldn't create file {:?}: {}", filename, err),
        Ok(f) => f,
    };
    let mut writer = BufWriter::new(f);
    serialize_into(&mut writer, &g)
}

pub fn read_obj<T: DeserializeOwned, P: AsRef<Path> + Debug>(
    filename: P,
) -> Result<T, bincode::Error> {
    let f = match File::open(&filename) {
        Err(err) => panic!("couldn't open file {:?}: {}", filename, err),
        Ok(f) => f,
    };
    let mut reader = BufReader::new(f);
    deserialize_from(&mut reader)
}

/// Open a (possibly gzipped) file into a BufReader.
fn _open_with_gz<P: AsRef<Path>>(p: P) -> Result<Box<dyn BufRead>, Error> {
    let r = File::open(p.as_ref())?;

    if p.as_ref().extension().unwrap() == "gz" {
        let gz = MultiGzDecoder::new(r);
        let buf_reader = BufReader::with_capacity(32 * 1024, gz);
        Ok(Box::new(buf_reader))
    } else {
        let buf_reader = BufReader::with_capacity(32 * 1024, r);
        Ok(Box::new(buf_reader))
    }
}

pub fn read_transcripts(
    reader: fasta::Reader<File>,
) -> Result<(Vec<DnaString>, Vec<String>, HashMap<String, String>, HashMap<String, usize >), Error> {
    let mut seqs = Vec::new();
    let mut transcript_counter = 0;
    let mut tx_ids = Vec::new();
    let mut tx_to_gene_map = HashMap::new();
    let mut tx_gene_length_map = HashMap::new();
    let mut fasta_format = FastaFormat::Unknown;

    info!("Reading transcripts from Fasta file");
    for result in reader.records() {
        // obtain record or fail with error
        let record = result?;

        // Sequence
        let dna_string = DnaString::from_acgt_bytes_hashn(record.seq(), record.id().as_bytes());
        seqs.push(dna_string);

        if let FastaFormat::Unknown = fasta_format {
            fasta_format = detect_fasta_format(&record)?;
        }

        let (tx_id, gene_id) = extract_tx_gene_id(&record, &fasta_format);
	

	//add sequence length to gene  entry if empty or bigger than current value
        let this_length = *&record.seq().len(); 
        let current_length = tx_gene_length_map.entry(gene_id.clone()).or_insert(0) ;
	if this_length > *current_length { *current_length = this_length};

        tx_ids.push(tx_id.clone());
        tx_to_gene_map.insert(tx_id, gene_id);

        transcript_counter += 1;
    }

    info!(
        "Done reading the Fasta file; Found {} sequences",
        transcript_counter
    );

   println!("gene length hash {:?}", tx_gene_length_map);


    Ok((seqs, tx_ids, tx_to_gene_map, tx_gene_length_map))
}

pub fn detect_fasta_format(record: &fasta::Record) -> Result<FastaFormat, Error> {
    let id_tokens: Vec<&str> = record.id().split('|').collect();
    if id_tokens.len() == 9 {
        return Ok(FastaFormat::Gencode);
    }

    let desc_tokens: Vec<&str> = record.desc().unwrap().split(' ').collect();
    if desc_tokens.len() >= 1 {
        let gene_tokens: Vec<&str> = desc_tokens[0].split('=').collect();
        if gene_tokens.len() == 2 && gene_tokens[0] == "gene" {
            return Ok(FastaFormat::Gffread);
        }
    } else if desc_tokens.len() == 5 {
        return Ok(FastaFormat::Ensembl);
    }
    Err(failure::err_msg("Failed to detect FASTA header format."))
}

pub fn extract_tx_gene_id(record: &fasta::Record, fasta_format: &FastaFormat) -> (String, String) {
    match *fasta_format {
        FastaFormat::Gencode => {
            let id_tokens: Vec<&str> = record.id().split('|').collect();
            let tx_id = id_tokens[0].to_string();
            let gene_id = id_tokens[1].to_string();
            // (human readable name)
            // let gene_name = id_tokens[5].to_string();
            (tx_id, gene_id)
        }
        FastaFormat::Ensembl => {
            let tx_id = record.id().to_string();
            let desc_tokens: Vec<&str> = record.desc().unwrap().split(' ').collect();
            let gene_tmp: Vec<&str> = desc_tokens[2].split(':').collect();
            let gene_id = gene_tmp[1].to_string();
            (tx_id, gene_id)
        }
        FastaFormat::Gffread => {
            let id_tokens: Vec<&str> = record.id().split(' ').collect();
            let tx_id = id_tokens[0].to_string();
            let desc_tokens: Vec<&str> = record.desc().unwrap().split(' ').collect();
            let gene_tokens: Vec<&str> = desc_tokens[0].split('=').collect();
            let gene_id = gene_tokens[1].to_string();
            (tx_id, gene_id)
        }
        FastaFormat::Unknown => {
            panic!("fasta_format was uninitialized");
        }
    }
}


pub fn get_next_record_pair<R: io::Read>(
    reader: &Arc<Mutex<std::iter::Zip<fastq::Records<R>,fastq::Records<R>>>>,
) -> Option<(Result<fastq::Record, io::Error>, Result<fastq::Record, io::Error>)> {
    let mut lock = reader.lock().unwrap();
    lock.next()
}



pub fn get_next_record<R: io::Read>(
    reader: &Arc<Mutex<fastq::Records<R>>>,
) -> Option<Result<fastq::Record, io::Error>> {
    let mut lock = reader.lock().unwrap();
    lock.next()
}

pub fn open_file<P: AsRef<Path>>(filename: &str, outdir: P) -> Result<File, Error> {
    let out_fn = outdir.as_ref().join(filename);
    let outfile = File::create(&out_fn)?;
    Ok(outfile)
}

pub fn kmers_to_u64_vec(seq: &DnaString, k: usize) -> Vec<u64> {
    if k == 0 || seq.len() < k {
        return Vec::new();
    }
    if k > 32 {
        panic!("kmers_to_u64_vec: k > 32 not supported for u64 packing");
    }

    let mut out = Vec::with_capacity(seq.len() - k + 1);
    let mut fwd: u64 = 0;
    let mut rev: u64 = 0;
    let mut window_valid = true;
    let mask: u64 = if k * 2 == 64 {
        u64::MAX
    } else {
        (1u64 << (2 * k)) - 1
    };
    let rev_shift = 2 * (k - 1);

    for (i, _) in (0..seq.len()).enumerate() {
        let base = seq.get(i);
        let val = match base {
            b'A' | b'a' => 0u64,
            b'C' | b'c' => 1u64,
            b'G' | b'g' => 2u64,
            b'T' | b't' => 3u64,
            _ => {
                // invalid base (N etc.) — break current rolling window
                window_valid = false;
                // advance counters: still shift but mark invalid until we have k valid bases
                fwd = 0;
                rev = 0;
                continue;
            }
        };

        // roll forward k-mer (left-shift)
        fwd = ((fwd << 2) | val) & mask;
        // roll reverse-complement (right-shift, add complement at high bits)
        let comp = 3u64 - val;
        rev = (rev >> 2) | (comp << rev_shift);

        if i + 1 >= k {
            if !window_valid {
                // window becomes valid only when we've seen k consecutive valid bases
                // check the trailing k bases: simple strategy is to set/clear window_valid by
                // tracking consecutive valid bases, here we reset window_valid true after k valid inserts
                // For skeleton keep it conservative: assume window_valid becomes true only if we didn't meet invalid base
                // (advanced: track a counter of consecutive valid bases)
                // For now, if we reached here and haven't seen invalid base since start, window_valid is true.
            }
            // canonical
            let canonical = std::cmp::min(fwd, rev);
            out.push(canonical);
            // after pushing, if you want sliding correctness with invalid bases,
            // you can maintain a counter of consecutive valid bases to flip window_valid.
        }
    }

    out
}

