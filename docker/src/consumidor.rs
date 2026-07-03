use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn conectar_buffer(endereco: &str, max_tentativas: u32) -> TcpStream {
    for tentativa in 1..=max_tentativas {
        match TcpStream::connect(endereco) {
            Ok(stream) => {
                println!("Conectado na tentativa {}", tentativa);
                return stream;
            }
            Err(e) => {
                println!("Tentativa {} falhou: {}", tentativa, e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    panic!("Não foi possível conectar ao buffer.");
}


fn main() {
    let host = env::var("BUFFER_HOST").unwrap();
    let port = env::var("BUFFER_PORT").unwrap();

    let endereco = format!("{}:{}", host, port);

    let total_itens = 5;
    let intervalo_ms = 800;

    let stream = conectar_buffer(&endereco, 1_000_000);

    let mut writer = stream.try_clone().expect("Erro ao clonar stream");
    let mut reader = BufReader::new(stream);

    for i in 1..=total_itens {
        if let Err(e) = writer.write_all(b"C\n") {
            eprintln!("[CONSUMIDOR] Erro ao solicitar item: {}", e);
            break;
        }

        let mut resposta = String::new();

        if reader.read_line(&mut resposta).is_ok() {
            println!(
                "[CONSUMIDOR] Item {}/{} consumido: {}",
                i,
                total_itens,
                resposta.trim()
            );
        } else {
            eprintln!("[CONSUMIDOR] Erro ao receber item.");
            break;
        }

        thread::sleep(Duration::from_millis(intervalo_ms));
    }

    println!("-------------------------------------------");
    println!("[CONSUMIDOR] Consumo finalizado.");
}