use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{BasicConsumeArguments, BasicPublishArguments, Channel, QueueBindArguments, QueueDeclareArguments};
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::{BasicProperties, DELIVERY_MODE_PERSISTENT};

use tokio::signal;
use tokio::sync::Notify;

use crate::request_handler::consumers::RabbitMQConsumer;

pub struct RabbitMQPublisher {
    connection: Connection,
    channel: Channel
}

impl RabbitMQPublisher {
    pub async fn new(
    host: &str,
    port: u16,
    username: &str,
    password: &str
    ) -> Result<Self, String> {
        let connection_arguments = OpenConnectionArguments::new(host, port, username, password);

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

        Ok(Self {
            connection: connection,
            channel: channel
        })
    }

    pub async fn publish_message(
       &self,
        message: &str
    ) -> Result<(), String> {
        if !self.connection.is_open() {
            return Err("Connection is not open".to_string());
        }

        if !self.channel.is_open() {
            return Err("Channel is not open".to_string());
        }
        println!("connection: {}, channel: {}", self.connection.to_string(), self.channel.to_string());

        let args = BasicPublishArguments::new("request_exchange", "response_exchange");

        self.channel
        .basic_publish(BasicProperties::default()
        .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
        .finish(),
        message.into(),
        args)
        .await
        .unwrap();

        Ok(())
    }
}

pub async fn start_consumer(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    routing_key: String,
    consumer_tag: &str
) -> Result<(), String> {
    // let connection_arguments =
    //     OpenConnectionArguments::try_from(addr.as_str()).expect("Could not parse RABBITMQ_URI");
    let connection_arguments = OpenConnectionArguments::new(host, port, username, password);

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
        .queue_declare(QueueDeclareArguments::default().queue("request_exchange".to_string()).durable(true).finish())
        .await
        .unwrap()
        .unwrap();

    //////////////////////////////////////////////////////////////////
    // start consumer with given name
    let args = BasicConsumeArguments::new("request_exchange", consumer_tag);

    let consumer = RabbitMQConsumer::new();
    channel
        .basic_consume(consumer, args)
        .await
        .unwrap();

    let guard = Notify::new();
    guard.notified().await;

    channel
    .basic_publish(BasicProperties::default(), b"asdf".to_vec(), BasicPublishArguments::default())
    .await
    .unwrap();


    return match signal::ctrl_c().await {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("Failed to listen for ctrl+c because of {}", err)),
    };
}

pub async fn start_publisher(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    routing_key: String,
    consumer_tag: &str
) -> Result<(), String> {
    // let connection_arguments =
    //     OpenConnectionArguments::try_from(addr.as_str()).expect("Could not parse RABBITMQ_URI");
    let connection_arguments = OpenConnectionArguments::new(host, port, username, password);

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
        .queue_declare(QueueDeclareArguments::default().queue("response_exchange".to_string()).durable(true).finish())
        .await
        .unwrap()
        .unwrap();

    //////////////////////////////////////////////////////////////////
    // start consumer with given name
    let args = BasicConsumeArguments::new("response_exchange", consumer_tag);

    let consumer = RabbitMQConsumer::new();
    channel
        .basic_consume(consumer, args)
        .await
        .unwrap();

    let guard = Notify::new();
    guard.notified().await;

    channel
    .basic_publish(BasicProperties::default(), b"asdf222".to_vec(), BasicPublishArguments::default())
    .await
    .unwrap();


    return match signal::ctrl_c().await {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("Failed to listen for ctrl+c because of {}", err)),
    };
}
