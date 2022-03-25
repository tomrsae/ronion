use core::crypto::ServerCrypto;
use std::{env, path::{Path, PathBuf}, fs::File, io::{Result, Read, Seek, Write}, hash::Hasher};

static PUBKEY_ENV: &str = "RO_PUBKEY";
static PRVKEY_ENV: &str = "RO_PRVKEY";
static PUBKEY_DEFAULT: &str = "keyfile.pub.rkf";
static PRVKEY_DEFAULT: &str = "keyfile.prv.rfk";

fn path_from_env(name: &str, default: &str) -> PathBuf {
    env::var(name).map_or_else(|_| PathBuf::from(default), PathBuf::from) 
}

fn file_from_env(name: &str, default: &str) -> Result<File> {
    let file_path = path_from_env(name, default);
    File::open(file_path)
}

fn read_from_file(mut file: File, buf: &mut [u8]) -> Result<()> {
    let end = file.seek(std::io::SeekFrom::End(0))?;
    file.seek(std::io::SeekFrom::Start(0))?;

    if end != 32 {
        panic!("keyfile was of invalid length");
    }

    file.read_exact(buf) 
}

pub fn gen_keys() -> Result<()> {
    let pub_path = path_from_env(PUBKEY_ENV, PUBKEY_DEFAULT);
    let prv_path = path_from_env(PRVKEY_ENV, PRVKEY_DEFAULT);
    
    let crypto = ServerCrypto::new();
    let pubkey = crypto.signing_public();
    let prvkey = crypto.to_bytes();

    File::create(pub_path)?.write(&pubkey)?;
    File::create(prv_path)?.write(&prvkey)?;

    Ok(())
}

pub fn read_public() -> [u8; 32] {
    let file = file_from_env("RO_PUBKEY", "keyfile.pub.rkf").expect("unable to open keyfile");
    let mut buf = [0u8; 32];
    read_from_file(file, &mut buf).expect("unable to read from file");
    buf
}

pub fn read_pair() -> [u8; 64] {
    let file = file_from_env("RO_PRVKEY", "keyfile.prv.rkf").expect("unable to open keyfile"); 
    let mut buf = [0u8; 64];
    read_from_file(file, &mut buf).expect("unable to read from file");
    buf
}
