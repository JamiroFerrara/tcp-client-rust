use tokio::{
    io::{AsyncWriteExt, BufReader, AsyncBufReadExt},
    net::TcpListener, sync::broadcast,
};

#[derive(Debug, Clone)]
struct Client {
    name: String,
    addr: std::net::SocketAddr
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    //channel for broadcasting messages
    let (tx, _rx) = broadcast::channel(10);

    loop {
        //This is a blocking call that waits for a new connection to be established
        let (mut socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // Create task to handle multiple clients
        tokio::spawn(async move {
            let (read, mut writer) = socket.split();

            let mut reader = BufReader::new(read);
            let mut line = String::new();

            writer.write_all("Welcome to the chat server!\n".as_bytes()).await.unwrap();
            writer.write_all("What is your name?\n".as_bytes()).await.unwrap();

            reader.read_line(&mut line).await.unwrap();
            let name = line.trim().to_string();
            line.clear();

            let client = Client { name, addr };

            //Chat mechanism
            loop {
                //Handle IO 
                tokio::select! {
                    // Read from client
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send((line.clone(), addr, client.clone())).unwrap();
                        line.clear();
                    }

                    // Write back to client
                    result = rx.recv() => {
                        let (msg, other_adder, client) = result.unwrap();
                        let full_message = client.name + ": " + &msg;

                        if addr != other_adder{
                            writer.write_all(full_message.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
