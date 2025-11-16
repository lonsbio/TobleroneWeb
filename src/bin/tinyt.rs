
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

        info!("Finished reading index!");

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
