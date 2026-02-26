// --------------------------------------1--------------------------------------------------
// use tokio::time::{Duration, sleep};

// async fn async_tasks(id: u32) {
//     println!("Task number {} is starting", id);
//     sleep(Duration::from_millis(id as u64 * 10)).await;
//     println!("Task number {} is completed", id);
// }

// fn main() {
//     tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(async {
//             let handle_a = tokio::spawn(async_tasks(1));
//             let handle_b = tokio::spawn(async_tasks(2));
//             let handle_c = tokio::spawn(async_tasks(3));
//             let handle_d = tokio::spawn(async_tasks(4));
//             let handle_e = tokio::spawn(async_tasks(5));

//             handle_a.await.unwrap();
//             handle_b.await.unwrap();
//             handle_c.await.unwrap();
//             handle_d.await.unwrap();
//             handle_e.await.unwrap();
//         });
// }

// --------------------------------------2--------------------------------------------------

// use std::sync::Arc;
// use tokio::{
//     task::JoinHandle,
//     time::{Duration, sleep},
// };

// fn spawn_with_shared_state(idx: usize, name: Arc<Vec<String>>) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         sleep(Duration::from_millis(50)).await;
//         println!("the name at index {} is {}", idx, name[idx]);
//     })
// }

// #[tokio::main]
// async fn main() {
//     let names = Arc::new(vec![
//         String::from("alpha"),
//         String::from("beta"),
//         String::from("gamma"),
//     ]);

//     let a = spawn_with_shared_state(0, Arc::clone(&names));
//     let b = spawn_with_shared_state(1, Arc::clone(&names));
//     let c = spawn_with_shared_state(2, Arc::clone(&names));

//     let _ = tokio::join!(a, b, c);
// }

// --------------------------------------3--------------------------------------------------

// use std::sync::{Arc, Mutex};
// use tokio::{
//     time::{sleep, Duration},
//     task::JoinHandle,
// };

// #[tokio::main]
// async fn main(){
//     let n = Arc::new(Mutex::new(0u32));
//     let mut handles : Vec<JoinHandle<()>> = vec![];

//     for _ in 0..10{

//         let n_clone = Arc::clone(&n);
//         let hand = tokio::spawn(async move{

//             sleep(Duration::from_millis(10)).await;
//             {
//                 let mut counter = n_clone.lock().unwrap();
//                 *counter +=1;
//             }
//             sleep(Duration::from_millis(10)).await;
//         });

//         handles.push(hand);
//     }

//     for hands in handles{
//         hands.await.unwrap();
//     }

//     println!("{}",*n.lock().unwrap());
// }

// --------------------------------------3--------------------------------------------------

// use tokio::{
//     sync::mpsc::{self, Sender},
//     task::JoinHandle,
//     time::{Duration, sleep},
// };

// fn message_from_producer(id: usize, tx_clone: Sender<String>) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         for i in 0..3 {
//             let msg = format!("Message {} from Producer {}", i, id);
//             if tx_clone.send(msg).await.is_err() {
//                 return;
//             }
//             sleep(Duration::from_millis(10)).await;
//         }

//     })
// }

// #[tokio::main]
// async fn main() {
//     let (tx, mut rx) = mpsc::channel::<String>(10);
//     let _ = message_from_producer(1, tx.clone());
//     let _ = message_from_producer(2, tx.clone());
//     let _ = message_from_producer(3, tx.clone());

//     drop(tx);

//     while let Some(msg) = rx.recv().await {
//         println!("{}", msg);
//     }
// }

// --------------------------------------4--------------------------------------------------

// use tokio::{
//     sync::mpsc,
//     time::{Duration, sleep},
// };

// #[tokio::main]
// async fn main() {
//     let (tx, mut rx) = mpsc::channel::<String>(1);
//     let tx_clone = tx.clone();
//     tokio::spawn(async move {
//         sleep(Duration::from_millis(50)).await;

//         match tx_clone.send(String::from("Late Data")).await {
//             Ok(_) => println!("Send succeeded"),
//             Err(e) => println!("Send FAILED: {}", e),
//         }
//     });

//     drop(tx);

//     loop {
//         tokio::select! {
//             msg = rx.recv() =>{
//                 println!("Late Data won, the message : {}", msg.unwrap());
//                 break;
//             }
//             _ = sleep(Duration::from_millis(20)) => {
//                 println!("Timeout! The query took too long.");
//             }

//         }
//     }
// }

// --------------------------------------5--------------------------------------------------

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::{Duration, sleep},
};

async fn read_or_timeout(stream: &mut TcpStream, timeout: Duration) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; 1024];

    let deadline = sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            result = stream.read(&mut buf) => {

                match result {
                    Ok(0)  => return Err("connection closed".into()),
                    Ok(n)  => return Ok(buf[..n].to_vec()),
                    Err(e) => return Err(e.to_string()),
                }
            }

            _ = &mut deadline => {

                return Err("timed out".into());
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listener bound to {}", addr);

    let server_task = tokio::spawn(async move {
        let (_socket, peer) = listener.accept().await.unwrap();
        println!("[server] Scenario A: accepted connection from {}", peer);
        sleep(Duration::from_millis(200)).await;
        println!("[server] Scenario A: done holding");

        let (mut socket, peer) = listener.accept().await.unwrap();
        println!("[server] Scenario B: accepted connection from {}", peer);
        sleep(Duration::from_millis(5)).await;
        socket.write_all(b"hello beast").await.unwrap();
        println!("[server] Scenario B: wrote bytes");
        sleep(Duration::from_millis(50)).await;
    });

    let client_task = tokio::spawn(async move {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        println!("[client] Scenario A: connected, waiting with 20ms timeout...");
        let result = read_or_timeout(&mut stream, Duration::from_millis(20)).await;
        println!("[client] Scenario A result: {:?}", result);
        assert!(result.is_err(), "expected timeout, got data");

     
        sleep(Duration::from_millis(250)).await;

        let mut stream = TcpStream::connect(addr).await.unwrap();
        println!("[client] Scenario B: connected, waiting with 100ms timeout...");
        let result = read_or_timeout(&mut stream, Duration::from_millis(100)).await;

        println!("[client] Scenario B result: {:?}", result);
        if let Ok(bytes) = &result {
            println!(
                "[client] Scenario B as string: {}",
                String::from_utf8_lossy(bytes)
            );
        }
        assert!(result.is_ok(), "expected data, got timeout");
    });

    tokio::join!(server_task, client_task).0.unwrap();
    println!("all done");
}
