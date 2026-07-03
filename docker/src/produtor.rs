use rand::Rng;
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
    let total_itens = 5;
    let intervalo_ms = 500;

    let endereco = format!("{}:{}", host, port);

    
    let stream = conectar_buffer(&endereco, 10000000);
    let mut writer = stream.try_clone().expect("Falha ao clonar stream");
    let mut reader = BufReader::new(stream);

    let mut rng = rand::thread_rng();

    for i in 1..=total_itens {
        let numero: i32 = rng.gen_range(1..=1000);

        match writer.write_all(format!("{}\n", numero).as_bytes()) {
            Ok(_) => {
                let mut resposta = String::new();
                if reader.read_line(&mut resposta).is_ok() {
                    println!("[PRODUTOR] Item {}/{} produzido: {}", i, total_itens, numero);
                }
            }
            Err(e) => {
                eprintln!("[PRODUTOR] Erro ao enviar item {}: {}. Erro: {}", i, numero, e);
                break;
            }
        }

        thread::sleep(Duration::from_millis(intervalo_ms));
    }

    println!("-------------------------------------------");
    println!("[PRODUTOR] Produção finalizada. {} itens produzidos.", total_itens);
}
