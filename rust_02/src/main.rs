use clap::Parser;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// HexTool – Read & Write binary files in hexadecimal
#[derive(Parser, Debug)]
#[command(name = "hextool", about = "Read and write binary files in hex")]
struct Args {
    /// Target file
    #[arg(short = 'f', long)]
    file: String,

    /// Read mode: display bytes as hex dump
    #[arg(short = 'r', long)]
    read: bool,

    /// Write mode: hex string to write
    #[arg(short = 'w', long)]
    write: Option<String>,

    /// Offset in bytes (decimal or 0x hex)
    #[arg(short = 'o', long, default_value = "0")]
    offset: String,

    /// Number of bytes to read
    #[arg(short = 's', long, default_value_t = 16)]
    size: usize,
}

fn parse_offset(s: &str) -> io::Result<u64> {
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid hex offset"))
    } else {
        s.parse::<u64>().map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid decimal offset"))
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let offset = parse_offset(&args.offset)?;
    let path = Path::new(&args.file);

    if args.read {
        read_mode(path, offset, args.size)?;
    }

    if let Some(hex_str) = args.write {
        write_mode(path, offset, &hex_str)?;
    }

    Ok(())
}

fn read_mode(path: &Path, offset: u64, size: usize) -> io::Result<()> {
    let mut file = File::open(path)?;

    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = vec![0u8; size];
    let bytes_read = file.read(&mut buffer)?;

    print_hex_dump(offset, &buffer[..bytes_read]);
    Ok(())
}

fn write_mode(path: &Path, offset: u64, hex_str: &str) -> io::Result<()> {
    // Convert hex string → bytes
    let bytes = hex_to_bytes(hex_str)?;

    let mut file = File::options().read(true).write(true).create(true).open(path)?;
    file.seek(SeekFrom::Start(offset))?;

    file.write_all(&bytes)?;

    println!("Writing {} bytes at offset 0x{:08X}", bytes.len(), offset);
    println!("Hex: {}", hex_str);
    print!("ASCII: ");
    for b in &bytes {
        print!("{}", ascii_or_dot(*b));
    }
    println!("\n✓ Successfully written");

    Ok(())
}

fn hex_to_bytes(s: &str) -> io::Result<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Hex string length must be even"));
    }
    let mut bytes = Vec::new();
    for i in (0..s.len()).step_by(2) {
        let byte_str = &s[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid hex byte"))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn print_hex_dump(start_offset: u64, bytes: &[u8]) {
    print!("{:08X}: ", start_offset);
    for b in bytes {
        print!("{:02X} ", b);
    }
    print!("|");
    for b in bytes {
        print!("{}", ascii_or_dot(*b));
    }
    println!("|");
}

fn ascii_or_dot(b: u8) -> char {
    match b {
        0x20..=0x7E => b as char,
        _ => '.',
    }
}
