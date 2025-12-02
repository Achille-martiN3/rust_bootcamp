use clap::{Parser, Subcommand};
use rand::Rng;
use std::fmt::Write as FmtWrite;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::time::Duration;

///  Diffie-Hellman key generation
#[derive(Parser, Debug)]
#[command(name = "streamchat", about = "Stream cipher chat with Diffie-Hellman key generation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Server {
        port: u16,
    },
    Client {
        
        addr: String,
    },
}

// Diffie–Hellman parameters 

const P: u64 = 0xD87FA3E291B4C7F3; // 64-bit prime (public)
const G: u64 = 2;                   // generator (public)

//  Keystream (LCG) 

struct Keystream {
    state: u32,
}

impl Keystream {
    fn from_secret(secret: u64) -> Self {
        let s = (secret as u32) ^ ((secret >> 32) as u32);
        Keystream { state: s }
    }

    fn next_u32(&mut self) -> u32 {
        const A: u32 = 1103515245;
        const C: u32 = 12345;
        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }

    fn next_byte(&mut self) -> u8 {
        (self.next_u32() >> 24) as u8 
    }

    fn xor_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| b ^ self.next_byte()).collect()
    }

    fn preview_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for _ in 0..n {
            out.push(self.next_byte());
        }
        out
    }
}

//Utilitaires 

fn hex_u64(v: u64) -> String {
    format!("{:016X}", v)
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut s = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        write!(&mut s, "{:02X}", b).unwrap();
    }
    s
}

fn read_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }
    Ok(input)
}

fn pow_mod(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result: u128 = 1;
    let mut b: u128 = (base % modulus) as u128;
    let m: u128 = modulus as u128;

    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * b) % m;
        }
        b = (b * b) % m;
        exp >>= 1;
    }

    result as u64
}

fn send_u64(stream: &mut TcpStream, v: u64) -> io::Result<()> {
    let buf = v.to_be_bytes();
    stream.write_all(&buf)
}

fn recv_u64(stream: &mut TcpStream) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

fn send_encrypted(stream: &mut TcpStream, ks: &mut Keystream, plain: &[u8]) -> io::Result<()> {
    println!("[ENCRYPT]");
    println!("Plain: {}", hex_bytes(plain));

    let cipher = ks.xor_bytes(plain);
    println!("Cipher: {}", hex_bytes(&cipher));

    let len = cipher.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&cipher)?;
    stream.flush()?;
    println!("[NETWORK] Sent {} bytes", cipher.len());
    Ok(())
}

fn recv_encrypted(stream: &mut TcpStream, ks: &mut Keystream) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut cipher = vec![0u8; len];
    stream.read_exact(&mut cipher)?;
    println!("[NETWORK] Received {} bytes", len);
    println!("[DECRYPT] Cipher: {}", hex_bytes(&cipher));

    let plain = ks.xor_bytes(&cipher);
    println!("Plain: {}", hex_bytes(&plain));
    Ok(plain)
}

//Serveur 

fn run_server(port: u16) -> io::Result<()> {
    println!("[SERVER] Listening on 0.0.0.0:{port}");
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    let (mut stream, addr) = listener.accept()?;
    println!("[SERVER] Client connected from {addr}");

    stream.set_read_timeout(Some(Duration::from_secs(120)))?;

    println!("\n[DH] Starting key exchange...");
    println!("[DH] Using hardcoded parameters:");
    println!("p = {}", hex_u64(P));
    println!("g = {}", G);

    let mut rng = rand::thread_rng();
    let private_key: u64 = loop {
        let k: u64 = rng.r#gen();
        if k % P != 0 {
            break k;
        }
    };
    let public_key = pow_mod(G, private_key, P);

    println!("[DH] Our private key  = {}", hex_u64(private_key));
    println!("[DH] Our public  key  = g^private mod p = {}", hex_u64(public_key));

    println!("[NETWORK] Sending our public key (8 bytes)...");
    send_u64(&mut stream, public_key)?;
    println!("[NETWORK] Waiting for their public key (8 bytes)...");
    let their_public = recv_u64(&mut stream)?;
    println!("[DH] Received their public key = {}", hex_u64(their_public));

    println!("\n[DH] Computing shared secret...");
    println!("Formula: secret = (their_public)^our_private mod p");
    let secret = pow_mod(their_public, private_key, P);
    println!("secret = ({})^({}) mod p = {}", hex_u64(their_public), hex_u64(private_key), hex_u64(secret));

    println!("\n[VERIFY] Shared secret computed.");
    let mut ks = Keystream::from_secret(secret);
    println!("[STREAM] Generating keystream from secret...");
    println!("Algorithm: LCG (a=1103515245, c=12345, m=2^32)");
    println!("Seed: {}", hex_u64(secret));
    let preview = ks.preview_bytes(8);
    println!("Keystream: {} ...", hex_bytes(&preview));
    println!("\n✓ Secure channel established!\n");

    let msg = read_line("[CHAT] Type message: ")?;
    let msg_bytes = msg.as_bytes();

    send_encrypted(&mut stream, &mut ks, msg_bytes)?;

    let reply = recv_encrypted(&mut stream, &mut ks)?;
    let reply_str = String::from_utf8_lossy(&reply);
    println!("[CLIENT] {}", reply_str);

    println!("\n[TEST] Round-trip done (server sent, client replied).");
    Ok(())
}

//  Client 

fn run_client(addr: &str) -> io::Result<()> {
    println!("[CLIENT] Connecting to {addr}...");
    let mut stream = TcpStream::connect(addr)?;
    println!("[CLIENT] Connected!");

    stream.set_read_timeout(Some(Duration::from_secs(120)))?;

    println!("\n[DH] Starting key exchange...");
    println!("[DH] Using hardcoded parameters:");
    println!("p = {}", hex_u64(P));
    println!("g = {}", G);

    let mut rng = rand::thread_rng();
    let private_key: u64 = loop {
        let k: u64 = rng.r#gen();
        if k % P != 0 {
            break k;
        }
    };
    let public_key = pow_mod(G, private_key, P);

    println!("[DH] Our private key  = {}", hex_u64(private_key));
    println!("[DH] Our public  key  = g^private mod p = {}", hex_u64(public_key));

    println!("[NETWORK] Waiting for server public key (8 bytes)...");
    let their_public = recv_u64(&mut stream)?;
    println!("[DH] Received their public key = {}", hex_u64(their_public));

    println!("[NETWORK] Sending our public key (8 bytes)...");
    send_u64(&mut stream, public_key)?;

    println!("\n[DH] Computing shared secret...");
    println!("Formula: secret = (their_public)^our_private mod p");
    let secret = pow_mod(their_public, private_key, P);
    println!("secret = ({})^({}) mod p = {}", hex_u64(their_public), hex_u64(private_key), hex_u64(secret));

    println!("\n[VERIFY] Shared secret computed.");
    let mut ks = Keystream::from_secret(secret);
    println!("[STREAM] Generating keystream from secret...");
    println!("Algorithm: LCG (a=1103515245, c=12345, m=2^32)");
    println!("Seed: {}", hex_u64(secret));
    let preview = ks.preview_bytes(8);
    println!("Keystream: {} ...", hex_bytes(&preview));
    println!("\n✓ Secure channel established!\n");

    let incoming = recv_encrypted(&mut stream, &mut ks)?;
    let incoming_str = String::from_utf8_lossy(&incoming);
    println!("[SERVER] {}", incoming_str);

    let reply = read_line("[CHAT] Type message: ")?;
    let reply_bytes = reply.as_bytes();

    send_encrypted(&mut stream, &mut ks, reply_bytes)?;

    println!("\n[TEST] Round-trip done (server received, client replied).");
    Ok(())
}

//  main 

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => run_server(port),
        Commands::Client { addr } => run_client(&addr),
    }
}
