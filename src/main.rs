use std::env;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::thread;
use hex;
use sha2::{Sha256, Digest};
use num_cpus;

fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();
  if args.len() != 4 {
    println!("Three arguments required in this order: filename firstchars(multiple of 2 in hex) lastchars(multiple of 2 in hex)");
    return Ok(());
  }

  let filename = &args[1];
  let firstchars = &args[2];
  let lastchars = &args[3];

  println!("Hashing {} and trying to append data to match {} ... {}", filename, firstchars, lastchars);

  let firstbytes = hex::decode(firstchars).expect("first bytes didn't unwrap, are they a multiple of 2 hex characters?");
  let lastbytes = hex::decode(lastchars).expect("last bytes didn't unwrap, are they a multiple of 2 hex characters?");

  let mut file = fs::File::open(filename)?;
  let mut data = Vec::new();
  file.read_to_end(&mut data)?;

  println!("Hash of input: {:x}", Sha256::digest(&data));

  let cpucount = num_cpus::get();
  let mut threads = Vec::new();
  for threadid in 0..cpucount {
    //for those unfamiliar with rust, these have to be cloned
    //in the loop because they are references to the object
    //using them directly would cause concurrency issues
    let data = data.clone();
    let firstbytes = firstbytes.clone();
    let lastbytes = lastbytes.clone();

    //for those unfamiliar with rust, the "move" keyword indicates
    //to the compiler that all references should be moved such that
    //they are owned by the new thread
    let h = thread::spawn(move || -> io::Result<()> {
      //each thread will look at its own vector size
      let mut vecsize = threadid+1;
      'outer: loop {
        //this creates an vector of `vecsize` zeroes
        let mut test: Vec<u8> = vec![0;vecsize];
        
        println!("Trying vector size: {}",test.len());

        'inner: loop {
          assert!(vecsize==test.len(), "test len is {}", test.len());
          'outerincrement: for i in 0..vecsize {
            //rust additions panic on overflow in debug mode
            test[i] = test[i].wrapping_add(1);
            if test[i] == 0 {
              if i+1==vecsize {
                //need to increase vector size
                vecsize += cpucount;
                continue 'outer;
              }
              continue 'outerincrement;
            }
            break 'outerincrement;
          }
          let mut testdata = data.clone();
          testdata.append(& mut test.clone());

          //now calculate the hash of the new vector
          //and check if it's what we want
          let hash = Sha256::digest(&testdata);
          for y in 0..firstbytes.len() {
            if hash[y] != firstbytes[y] {
              continue 'inner;
            }
          }
          for y in 0..lastbytes.len() {
            if hash[31-y] != lastbytes[y] {
              continue 'inner;
            }
          }
          println!("Hash of test:  {:x}", hash);
          fs::write("out",testdata).expect("Unable to write file");
          std::process::exit(0);
        }
      }
    });
    threads.push(h);
  }

  for t in threads.into_iter() {
    t.join().ok();
  }

  Ok(())
}
