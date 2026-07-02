use std::collections::VecDeque;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct Semaforo {
    contador: Mutex<i32>,
    condvar: Condvar,
}

impl Semaforo {

    fn novo(valor_inicial: i32) -> Self {
        Semaforo {
            contador: Mutex::new(valor_inicial),
            condvar: Condvar::new(),
        }
    }


    fn wait(&self) {
        let mut valor = self.contador.lock().unwrap();
        while *valor <= 0 {

            valor = self.condvar.wait(valor).unwrap();
        }
        *valor -= 1;
    }

    fn signal(&self) {
        let mut valor = self.contador.lock().unwrap();
        *valor += 1;
        self.condvar.notify_one();
    }
}

struct BufferCompartilhado {
    fila: Mutex<VecDeque<i32>>,
    vagas: Semaforo, 
    itens: Semaforo,
    capacidade: i32,
}

impl BufferCompartilhado {
    fn novo(capacidade: i32) -> Self {
        BufferCompartilhado {
            fila: Mutex::new(VecDeque::new()),
            vagas: Semaforo::novo(capacidade), 
            itens: Semaforo::novo(0),   
            capacidade : capacidade,
        }
    }


    fn produzir(&self, valor: i32) {

        self.vagas.wait();

        {
            let mut fila = self.fila.lock().unwrap();
            fila.push_back(valor);
            println!(
                "[BUFFER] Produzido: {} | Buffer: {}/{}",
                valor,
                fila.len(),
                self.capacidade
            );
        } 
        self.itens.signal();
    }

    fn consumir(&self) -> i32 {
        
        self.itens.wait();

        let valor;
        {
            let mut fila = self.fila.lock().unwrap();
            valor = fila.pop_front().expect("Fila não deveria estar vazia após wait(itens)");
            println!(
                "[BUFFER] Consumido: {} | Buffer: {}/{}",
                valor,
                fila.len(),
                self.capacidade
            );
        } 

        self.vagas.signal();

        valor
    }
}


fn tratar_conexao(stream: TcpStream, buffer: Arc<BufferCompartilhado>) {

    let reader = BufReader::new(stream.try_clone().expect("Falha ao clonar stream"));
    let mut writer = stream;

    for linha_result in reader.lines() {
        match linha_result {
            Ok(linha) => {
                let comando = linha.trim().to_string();

                if let Ok(numero) = comando.parse::<i32>() {
                    buffer.produzir(numero);
                    let _ = writer.write_all(b"OK\n");
                } else if comando == "C" {
                    let valor = buffer.consumir();
                    let _ = writer.write_all(format!("{}\n", valor).as_bytes());
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }

}

fn main() {

    let capacidade = 50;
    let porta = env::var("BUFFER_PORT").unwrap();
    let endereco = format!("0.0.0.0:{}", porta);

    let buffer = Arc::new(BufferCompartilhado::novo(capacidade));
    let listener = TcpListener::bind(&endereco).expect("Não foi possível iniciar o servidor TCP");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let buffer = Arc::clone(&buffer);
                thread::spawn(move || {
                    tratar_conexao(stream, buffer);
                });
            }
            Err(e) => {
                eprintln!("[BUFFER] Erro ao aceitar conexão: {}", e);
            }
        }
    }
}
