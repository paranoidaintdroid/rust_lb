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

use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinHandle,
    time::{Duration, sleep},
};

fn message_from_producer(id: usize, tx_clone: Sender<String>) -> JoinHandle<()> {
    tokio::spawn(async move {
        for i in 0..3 {
            let msg = format!("Message {} from Producer {}", i, id);
            if tx_clone.send(msg).await.is_err() {
                return;
            }
        }
        sleep(Duration::from_millis(10)).await;
    })
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(10);
    let tx_clone = tx.clone();
    let _ = message_from_producer(1, tx_clone);
    let tx_clone = tx.clone();
    let _ = message_from_producer(2, tx_clone);
    let tx_clone = tx.clone();
    let _ = message_from_producer(3, tx_clone);

    drop(tx);

    while let Some(msg) = rx.recv().await {
        println!("{}", msg);
    }
}
