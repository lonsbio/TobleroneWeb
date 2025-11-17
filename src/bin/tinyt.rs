
// Copyright (c) 2018 10x Genomics, Inc. All rights reserved.
// Copyright (c) 2021 Andrew Lonsdale tinyt version

use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};
//use serde::Deserialize;
use env_logger::fmt::Target;
use std::io::Write;
use bio::io::{fasta, fastq};
use docopt::Docopt;
use failure::Error;
use std::{env, fs};
use std::{path::PathBuf, str};
use boomphf::Mphf;
use serde::{Serialize, Deserialize};

use tinyt::{
    build_index::{build_index,export_wasm_index,WasmRuntimeIndex, WasmIndex,IndexLike},
    pseudoaligner,
    // pseudoaligner::process_reads,
   pseudoaligner::{process_reads, Pseudoaligner},

};
use tinyt::{config, utils};
use crate::config::{LEFT_EXTEND_FRACTION, READ_COVERAGE_THRESHOLD, DEFAULT_ALLOWED_MISMATCHES, TRIM_VAL};
use debruijn::dna_string::{DnaString, DnaStringSlice};
use std::collections::{HashMap, HashSet};



const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
tinyt

Usage:
  tinyt index [--num-threads=<n>] [--wasm] --index=<index> <ref-fasta>
  tinyt map [--num-threads=<n>] [--read-length=<r>] [--trim-size=<t>] [--skip-trim] [--mismatch=<m>] [--output=<file>]  [--wasm]  --index=<index>  <reads-fastq> [<reads-pair-fastq>]
  tinyt -h | --help | -v | --version

Options:
  -n --num-threads N  Number of worker threads [default: 2]
  -w --wasm           Create or read index in WASM compatible format
  -i --index INDEX    Index file to write to or read from
  -t --trim-size T    Size of base pairs to trim when checking unique read matches [default: 5] 
  -s --skip-trim      Skip the trim read check for unqiue read matches
  -m --mismatch M     Number of allowed mismatches for per read [default: 2]
  -r --read-length R  Provide read length for depth estimation
  -o --output FILE    Output results to file instead of stdout
  -h --help           Show this screen.
  -v --version        Show version.
";

#[derive(Clone, Debug, Deserialize)]
struct Args {
    arg_ref_fasta: String,
    flag_index: String,
    arg_reads_fastq: String,
    arg_reads_pair_fastq: String,
    flag_output: Option<String>,
    flag_num_threads: usize,

    flag_wasm: bool,

    cmd_index: bool,

    cmd_map: bool,
    flag_trim_size: usize,
    flag_mismatch: usize,
    flag_skip_trim: bool,
    flag_read_length: Option<usize>,


    flag_version: bool,
    flag_v: bool,
}

fn main() -> Result<(), Error> {
    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());



   // let level = log::LevelFilter::Info;
//      pretty_env_logger::formatted_timed_builder().target(Target::Stdout).init();
 // initialize logger
     if env::var_os("RUST_LOG").is_none() {                                                                                                                                                                                                                                      
       env::set_var("RUST_LOG", "tinyt=info");                                                                                                                                                                                                                             
     }   
    pretty_env_logger::init_timed();

    

	if args.flag_version || args.flag_v {
        println! {"{} {}", PKG_NAME, PKG_VERSION};
        return Ok(());
    }

 if args.flag_skip_trim && args.flag_trim_size != 2 {
        warn! {"--trim-size has no effect when --skip-trim used"};
    }
 if args.flag_trim_size ==0 {
        warn! {"--trim-size of 0 implies --skip-trim"};
       args.flag_skip_trim = true; 
    }

if args.flag_trim_size <= args.flag_mismatch {
        warn! {"--trim-size less than or equal to mismatch may be ineffective"};
    }

    debug!("Command line args:\n{:?}", args);



    if args.cmd_index {
        info!("Building index from fasta: {}",&args.arg_ref_fasta);
        let fasta = fasta::Reader::from_file(&args.arg_ref_fasta)?;
        let (seqs, tx_names, tx_gene_map, gene_length_map) = utils::read_transcripts(fasta)?;
                    info!("Building native index");

                    let index =             build_index::<config::KmerType>(&seqs, &tx_names, &tx_gene_map,&gene_length_map,  args.flag_num_threads)?;


        if args.flag_wasm {    
        let wasm_idx = export_wasm_index::<config::KmerType>(&index);
        let wasm_path = format!("{}.wasm.idx", &args.flag_index);
        utils::write_obj(&wasm_idx, &wasm_path)?;
        info!("WASM index written to {}", wasm_path);
              
        } 
      

       
        info!("Finished building index!");

        info!("Writing index to disk");
        utils::write_obj(&index, &args.flag_index)?;
        info!("Finished writing index!");
        info!("Writing minimial index to disk");
        let bundle = export_minimal_index(&index);
        write_minimal_index("testindex.minidx", &bundle);
        info!("Finished writing minimial index to disk");    

        info!("Total equivalence classes: {}", index.dbg.len() );

        use debruijn::Mer;
        use std::fs::File;
        use std::io::Write;


        // output index statistics to "args.arg_index.index.ec.csv"
        let mut w = File::create(format!("{}.ec.csv", &args.flag_index)).unwrap();
        let mut unique_ec = 0;
            //println!("EC,SeqLength,TranscriptCount,TranscriptNames");
            writeln!(&mut w,"EC,SeqLength,TranscriptCount,TranscriptNames").unwrap();
        for e in index.dbg.iter_nodes() {
            let eqid = e.data();
            let eq = &index.eq_classes[*eqid as usize];
            if eq.len() == 1 {
		unique_ec += 1;
            writeln!(&mut w,"EC{},{},{},{:?}", e.node_id, e.sequence().len(), eq.len(), index.tx_names[eq[0] as usize]).unwrap();
	    } else {

            //let nameslist = eq.iter().map(|x| &index.tx_names[*x as usize]).collect().join(",");
            let mut nameslist = String::new();
            for n in eq {

               //println!("{}",index.tx_names[*n as usize]);
            if !nameslist.is_empty() {
            nameslist.push(',');
            }
             nameslist.push_str(&index.tx_names[*n as usize]);


           //    &nameslist.append(index.tx_names[*n as usize]);
            }

            writeln!(&mut w,"EC{},{},{},{:?}", e.node_id, e.sequence().len(), eq.len(),nameslist).unwrap();
            //debug!("EC{}\t{}\t{}\t{:?}", e.node_id, e.sequence().len(), eq.len(),nameslist);

            }

        
        }

            info!("Unique equivalence classes: {}", unique_ec);


    } else if args.cmd_map {
        info!("Reading index from disk");
        if args.flag_wasm  {
            info!("Reading WASM compatible index");
        } else {
            info!("Reading native index");
        }
        // let index =if args.flag_wasm  {
        //     let wasm_idx = utils::read_obj(args.flag_index)?;
        //     WasmRuntimeIndex::from_wasm_index(wasm_idx)
        // } else {
        //     utils::read_obj(args.flag_index)?
            
        // };


         let index_box: Box<dyn IndexLike> = if args.flag_wasm {
            // read the compact WasmIndex (must implement Deserialize)
            let wasm_idx: WasmIndex = utils::read_obj(&args.flag_index)?;
            Box::new(WasmRuntimeIndex::from_wasm_index(wasm_idx))
        } else {
            // read the full native Pseudoaligner (concrete type)
             let native_idx: Pseudoaligner<config::KmerType> = utils::read_obj(&args.flag_index)?;
                    // let native_idx  = utils::read_obj(&args.flag_index)?;

            Box::new(native_idx)
        };

        // let index = index_box;


        // debug
    use std::cmp::min;
    // If we loaded a WasmRuntimeIndex, do extra diagnostics
    if args.flag_wasm {
        let wasm_idx: WasmIndex = utils::read_obj(&args.flag_index)?;
         let wasm_rt = WasmRuntimeIndex::from_wasm_index(wasm_idx);


        info!("WASM index k = {}", wasm_rt.k);
        info!("WASM kmers (flat) count = {}", wasm_rt.lookup.len());
        info!("WASM eq_classes count = {}", wasm_rt.eq_classes.len());

        // print a few entries from the lookup
        for (i, (kmer, (node, off))) in wasm_rt.lookup.iter().take(8).enumerate() {
            // info!("sample lookup #{} kmer=0x{:x} node={} off={}", i, kmer, node, off);
        }

        // quick test: open reads and test first record's kmers
        if let Ok(mut reader) = bio::io::fastq::Reader::from_file(&args.arg_reads_fastq) {
            if let Some(Ok(rec)) = reader.records().next() {
                let seq = tinyt::utils::dna_from_fastq_record(&rec); // helper below if you don't have one
                let kmers = tinyt::utils::kmers_to_u64_vec(&seq, wasm_rt.k as usize);
                // info!("First read length {} produced {} kmers (k={})", seq.len(), kmers.len(), wasm_rt.k);
                // for (i, kmer) in kmers.iter().take(min(12, kmers.len())).enumerate() {
                //     match wasm_rt.lookup.get(kmer) {
                //         Some((node_id, off)) => info!("read kmer[{}]=0x{:x} FOUND -> node {} off {}", i, kmer, node_id, off),
                //         None => info!("read kmer[{}]=0x{:x} MISS", i, kmer),
                //     }
                // }
            }
        }
    } else {
        info!("Loaded native index ");
       let index: Pseudoaligner<config::KmerType> = utils::read_obj(args.flag_index.clone())?;
 info!("Native index: tx count = {}", index.tx_names.len());
     info!("Native index: eq_classes count = {}", index.eq_classes.len());
       // dbg.len() returns number of nodes/entries in DBG
      info!("Native index: dbg size = {}", index.dbg.len());
        // Probe first read (if available) and run a single map_read to see hits/misses
        if let Ok(mut reader) = fastq::Reader::from_file(&args.arg_reads_fastq) {
            if let Some(Ok(rec)) = reader.records().next() {
                let seq_str = std::str::from_utf8(rec.seq()).unwrap_or("");
               let seq = DnaString::from_dna_string(seq_str);
                let kmers = utils::kmers_to_u64_vec(&seq, /* use index k (if exposed) fallback to 20 */ 20);
                info!("First read length {} produced {} kmers (probe)", seq.len(), kmers.len());

                // call the native Pseudoaligner map_read (uses concrete type)
                match index.map_read(&seq, args.flag_mismatch) {
                    Some((eq, cov, mm, rl)) => {
                        info!("map_read probe -> eq.len={} cov={} mm={} readlen={}", eq.len(), cov, mm, rl);
                        if !eq.is_empty() {
                            let name = &index.tx_names[eq[0] as usize];
                            info!("map_read probe -> first tx hit = {}", name);
                        }
                    }
                   None => info!("map_read probe -> no hit"),
                }
            }
        }
    }
        //
        info!("Finished reading index!");


        // --- diagnostic: load both native and wasm index for side-by-side comparison ---
        let native_idx: Pseudoaligner<config::KmerType> = utils::read_obj("compiletest.idx")?;
        let wasm_path = format!("{}.wasm.idx", &args.flag_index);
        let wasm_rt = if std::path::Path::new(&wasm_path).exists() {
            let wasm_idx: WasmIndex = utils::read_obj(&wasm_path)?;
            WasmRuntimeIndex::from_wasm_index(wasm_idx)
        } else {
            // no wasm index available - build one from native for diagnostics
            let wasm_idx = tinyt::build_index::export_wasm_index::<config::KmerType>(&native_idx);
            WasmRuntimeIndex::from_wasm_index(wasm_idx)
        };
// {
//     use debruijn::dna_string::DnaString;
//     use bio::io::fastq;
//     use std::cmp::min;

//     let mut reader = fastq::Reader::from_file(&args.arg_reads_fastq)?;
//     info!("Cross-debug: scanning for native-none/wasm-some and native-multi/wasm-unique (up to 200 examples)");

//     let mut printed_none_wasm_some = 0usize;
//     let mut printed_native_multi_wasm_unique = 0usize;
//     let mut total_reads = 0usize;
//     let mut examples_limit = 500usize;

//     for rec in reader.records() {
//         if printed_none_wasm_some + printed_native_multi_wasm_unique >= examples_limit { break; }
//         let rec = rec?;
//         total_reads += 1;
//         let id = rec.id().to_string();
//         let s = std::str::from_utf8(rec.seq()).unwrap_or("").to_string();
//         let seq = DnaString::from_dna_string(&s);

//         let native_opt = native_idx.map_read(&seq, args.flag_mismatch);
//         // info!("Cross-debug: read id={} native map_read result: {:?}", id, native_opt);
//         let wasm_opt = wasm_rt.map_read(&seq, args.flag_mismatch);
//         // info!("Cross-debug: read id={} wasm map_read result: {:?}", id, wasm_opt);
//         // Case A: native none, wasm some
//         if native_opt.is_none() && wasm_opt.is_some() && printed_none_wasm_some < 100 {
//             printed_none_wasm_some += 1;
//             let (weq, wcov, wmm, wrl) = wasm_opt.as_ref().unwrap();
//             // let (ne, ncov, nmm, nrl) = native_opt.as_ref().unwrap();
//             // info!("NATIVE-NONE n_eq_len={} n_cov={} n_mm={} n_rl={} / WASM-SOME id={} wasm_eq_len={} wasm_cov={} wasm_mm={} wasm_rl={}",
//                 // id, ne.len(), ncov, nmm, nrl,weq.len(), wcov, wmm, wrl);

//                 //   info!("CASE A NATIVE-NONE / WASM-SOME id={} wasm_eq_len={} wasm_cov={} wasm_mm={} wasm_rl={}",
//                 // id, weq.len(), wcov, wmm, wrl);


//             let k = wasm_rt.k as usize;
//             let kmers = utils::kmers_to_u64_vec(&seq, k);
//             let total_hits = kmers.iter().filter(|k| wasm_rt.lookup.contains_key(k)).count();
//             info!(" kmers={} total_hits={}", kmers.len(), total_hits);

//             let wasm_names: Vec<_> = weq.iter().filter_map(|&t| wasm_rt.tx_names.get(t as usize)).take(32).cloned().collect();
//             info!(" wasm tx_names (first 32) = {:?}", wasm_names);
//         }

//         // Case B: native unique , wasm unique
//         if let Some((ne, ncov, nmm, nrl)) = native_opt.as_ref() {
//             if ne.len() == 1 {
//                 if let Some((weq, wcov, wmm, wrl)) = wasm_opt.as_ref() {
//                     if weq.len() == 1  {
//                         printed_native_multi_wasm_unique += 1;
//                         info!("CASE B - NATIVE-unqiue / WASM-UNIQUE id={} native_eq_len={} wasm_eq_len={} native_cov={} wasm_cov={}",
//                             id, ne.len(), weq.len(), ncov, wcov);
//                         let k = wasm_rt.k as usize;
//                         let kmers = utils::kmers_to_u64_vec(&seq, k);
//                         let total_hits = kmers.iter().filter(|k| wasm_rt.lookup.contains_key(k)).count();
//                         info!("CASE B kmers={} total_hits={}", kmers.len(), total_hits);

//                         info!("CASE B native nmm{:?},nrl{:?}, wasm wmm {:?}, wrl, {:?}", nmm, nrl, wmm, wrl);




//                         let native_names: Vec<_> = ne.iter().filter_map(|&t| native_idx.tx_names.get(t as usize)).take(32).cloned().collect();
//                         let wasm_names: Vec<_> = weq.iter().filter_map(|&t| wasm_rt.tx_names.get(t as usize)).take(32).cloned().collect();
//                         info!("CASE B native tx_names (first 32) = {:?}", native_names);
//                         info!("CASE B wasm tx_names (first 32)   = {:?}", wasm_names);
//                     } else {
//                         info!("hello");
//                         // wasm multi - skip
//                            info!("CASE C - NATIVE-UNIQUE   / WASM-UNIQUE id={} native_eq_len={} wasm_eq_len={} native_cov={} wasm_cov={}",
//                             id, ne.len(), weq.len(), ncov, wcov);
//                 }
//                 } 
//                     // wasm none
//             }
//         }
//     }

//     info!("Cross-debug summary: total_reads={} native-none/wasm-some_printed={} native-multi/wasm-unique_printed={}",
//         total_reads, printed_none_wasm_some, printed_native_multi_wasm_unique);
// }
        // --- end diagnostics ---

        info!("Mapping reads from fastq");
        let reads = fastq::Reader::from_file(args.arg_reads_fastq)?;
    if args.arg_reads_pair_fastq == ""  {
        info!("Single end reads provided");
        process_reads::<config::KmerType>(reads,None, &*index_box, args.flag_output, args.flag_num_threads,!args.flag_skip_trim,args.flag_trim_size,args.flag_mismatch,args.flag_read_length)?;
    } else {
        info!("Paired end reads provided");
        let reads_pair = fastq::Reader::from_file(args.arg_reads_pair_fastq)?;
        process_reads::<config::KmerType>(reads,Some(reads_pair), &*index_box, args.flag_output, args.flag_num_threads,!args.flag_skip_trim,args.flag_trim_size,args.flag_mismatch,args.flag_read_length)?;
    }


    }

    info!("Done!");
    Ok(())
}
