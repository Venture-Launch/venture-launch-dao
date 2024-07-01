use std::env;

use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{BasicConsumeArguments, QueueDeclareArguments};
use amqprs::connection::{Connection, OpenConnectionArguments};
use tokio::signal;
use tokio::sync::Notify;

use crate::request_handler::consumers::RabbitMQConsumer;

pub async fn start(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    routing_key: String,
    consumer_tag: &str,
) -> Result<(), String> {
    // let connection_arguments =
    //     OpenConnectionArguments::try_from(addr.as_str()).expect("Could not parse RABBITMQ_URI");
    let connection_arguments = OpenConnectionArguments::new(host, port, username, username);

    let connection = Connection::open(&connection_arguments)
        .await
        .expect("Connection to RabbitMQ failed");

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    // open a channel on the connection
    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    // declare a queue
    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::default().queue(routing_key).finish())
        .await
        .unwrap()
        .unwrap();

    //////////////////////////////////////////////////////////////////
    // start consumer with given name
    let args = BasicConsumeArguments::new(&queue_name, consumer_tag);

    channel
        .basic_consume(RabbitMQConsumer::new(), args)
        .await
        .unwrap();

    let guard = Notify::new();
    guard.notified().await;

    return match signal::ctrl_c().await {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("Failed to listen for ctrl+c because of {}", err)),
    };
}
