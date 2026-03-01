use laser_cpf::{ParseOptions, read_cpf_v2};
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <cpf_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            std::process::exit(1);
        }
    };

    let reader = BufReader::new(file);
    match read_cpf_v2(reader, &ParseOptions::default()) {
        Ok((header, ephemeris)) => {
            println!("{:#?}", header);
            println!("{:#?}", ephemeris);
        }
        Err(e) => {
            eprintln!("Error parsing CPF file: {}", e);
            std::process::exit(1);
        }
    }
}
