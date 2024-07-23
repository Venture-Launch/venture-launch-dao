use dotenv::dotenv;
use std::error::Error;
use tokio;
pub mod multisig_utils;
pub mod dao_module;
pub mod request_handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let host = std::env::var("RABBIT_HOST").unwrap_or_else(|_| "localhost".into());

    let response_queue = std::env::var("RESPONSE_QUEUE").unwrap_or_else(|_| "response_queue".into());
    let requesr_queue = std::env::var("REQUEST_QUEUE").unwrap_or_else(|_| "request_queue".into());

    let req_exchange: String =
        std::env::var("REQUEST_EXCHANGE").unwrap_or_else(|_| "request_exchange".into());
    let resp_exchange: String =
        std::env::var("RESPONSE_EXCHANGE").unwrap_or_else(|_| "response_exchange".into());
    // let port = std::env::var("RABBIT_PORT").unwrap_or_else(|_| "localhost".into());

    let rabbit_consume = request_handler::processor::start_consumer(
        host.as_str(),
        5672,
        "guest",
        "guest",
        req_exchange,
        "request.consumer",
    );

    // let rabbit_publish    = request_handler::processor::start_publisher(
    //     host.as_str(),
    //     5672,
    //     "guest",
    //     "guest",
    //     resp_exchange,
    //     "response.consumer",
    // );

    // let _ = tokio::join!(rabbit_consume, rabbit_publish);
    let _ = tokio::join!(rabbit_consume);

    // env_logger::init();

    // let addr = std::env::var("RABBIT_DEFAULT_URL").unwrap_or_else(|_| "amqp://localhost".into());

    // let conn = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?;
    // println!("CONNECTED");

    // let channel = conn.create_channel().await?;
    // println!("CREATED CHANNEL");

    // // Declare an exchange
    // channel
    //     .exchange_declare(
    //         "hello",
    //         lapin::ExchangeKind::Direct,
    //         ExchangeDeclareOptions {
    //             durable: true, // Match the existing configuration
    //             ..ExchangeDeclareOptions::default()
    //         },
    //         FieldTable::default(),
    //     )
    //     .await?;
    // println!("Declared exchange");

    // // Declare a queue
    // let queue = channel
    //     .queue_declare(
    //         "hello",
    //         QueueDeclareOptions::default(),
    //         FieldTable::default(),
    //     )
    //     .await?;
    // println!("Declared queue {:?}", queue);

    // // Bind the queue to the exchange
    // channel
    //     .queue_bind(
    //         "hello",
    //         "hello",
    //         "hello",
    //         QueueBindOptions::default(),
    //         FieldTable::default(),
    //     )
    //     .await?;
    // println!("Bound queue to exchange");

    // // Start consuming messages from the queue
    // let mut consumer = channel
    //     .basic_consume(
    //         "hello",
    //         "my_consumer",
    //         BasicConsumeOptions::default(),
    //         FieldTable::default(),
    //     )
    //     .await?;

    // println!("WILL CONSUME");
    // tokio::spawn(async move {
    //     while let Some(delivery) = consumer.next().await {
    //         match delivery {
    //             Ok(delivery) => {
    //                 let message = std::str::from_utf8(&delivery.data).unwrap();
    //                 println!("Received [{}]", message);

    //                 delivery.ack(BasicAckOptions::default()).await.expect("ack");
    //             }
    //             Err(error) => {
    //                 eprintln!("Error in consumer: {:?}", error);
    //             }
    //         }
    //     }
    // });
    // println!(" [*] Waiting for messages. To exit press CTRL+C");
    // let guard = Notify::new();
    // guard.notified().await;
    Ok(())
}
