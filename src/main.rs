use anyhow::anyhow;
use chacha20poly1305::{
    aead::{stream, Aead, NewAead},
    XChaCha20Poly1305,
};
use rand::{rngs::OsRng, RngCore};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    env
};

mod gui;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please specify an action, e.g. 'help'");
        return;
    }

    let action = &args[1];

    if action == "help" {
        if args.len() < 3 {
            help_message("help");
            return;
        } else {
            let action = &args[2];
            help_message(action);
            return;
        }
    }

    if action == "encfnopass" {
        if args.len() < 4 {
            println!("Please specify a file to encrypt and an output file");
            return;
        } else {
            let file = &args[2];
            let output_file = &args[3];
            encfnopass(file, output_file);
            return;
        }
    }

    match action.as_str() {
        "gui" => run_gui(),
        _ => println!("Unknown action: {}", action),
    }
}

fn help_message(action: &str) {
    match action {
        "help" => println!("###############################\n\
        #                             #\n\
        #  Welcome to the Kylian CLI  #\n\
        #                             #\n\
        ###############################\n\
        \n\
        help - Show this message\n\
        encfnopass - Encrypt a single file\n\
        ###############################\n\
        - You can also use 'help <command>' to get more information about a command
        \n\
        "),
        "encf" => println!("This command will encrypt a single file using AES-256 and no password\n\
        Usage: kylian encfnopass <file> <output file>\n\
        "),
        _ => println!("Unknown action: {}", action),
    }
}

fn encfnopass(file: &str, output_file: &str) {

    if !check_file_exists(file) {
        println!("File: {} does not exist", file);
        return;
    } else {
        println!("File: {} exists", file);
    }
    encrypt_file_no_pass(file, output_file).unwrap();

    

}

fn check_file_exists(file: &str) -> bool {
    let path = Path::new(file).exists();
    return path;
}

fn encrypt_file_no_pass(input_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 19];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    // write key and nonce to file
    let mut key_file = File::create("key.txt")?;
    key_file.write_all(&key)?;
    let mut nonce_file = File::create("nonce.txt")?;
    nonce_file.write_all(&nonce)?;

    
    let aead = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream_encryptor = stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into());

    const BUFFER_LEN: usize = 500;
    let mut buffer = [0u8; BUFFER_LEN];

    let mut source_file = File::open(input_file)?;
    let mut dist_file = File::create(output_file)?;

    loop {
        let read_count = source_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let ciphertext = stream_encryptor
                .encrypt_next(buffer.as_slice())
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dist_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dist_file.write(&ciphertext)?;
            println!("Encryption complete, file: {} encrypted to: {}", input_file, output_file);
            break;
        }
    }


    Ok(())
}

fn run_gui() {
    gui::run_window();
}

