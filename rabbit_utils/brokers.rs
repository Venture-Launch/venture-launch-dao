use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use tracing::info;

pub struct Consumer<'a> {
    url: &'a str,
    channel_id: Option<u16>,
    queue_name: &'a str,
}

impl Consumer<'_> {
    pub fn create<'a>(url: &'a str, channel_id: Option<u16>, queue_name: &'a str) -> Consumer<'a> {
        Consumer {
            url,
            channel_id,
            queue_name,
        }
    }

    pub fn init(&self) {
        let mut connection =
            Connection::insecure_open(self.url).expect("Failed to connect to RabbitMQ");

        let channel = connection
            .open_channel(self.channel_id)
            .expect("Unable to open channel");

        let queue = channel
            .queue_declare(
                self.queue_name,
                QueueDeclareOptions {
                    durable: true, // Set to true if the queue is durable, otherwise set to false.
                    exclusive: false,
                    auto_delete: false,
                    arguments: Default::default(),
                },
            )
            .map_err(|e| eprintln!("Failed to declare queue: {}", e))
            .expect("Unable to create queue!");

        let binding = channel
            .queue_bind("queue", "queue", "hello", Default::default())
            .expect("test");

        info!(?binding, "binding");
        let consumer = queue
            .consume(ConsumerOptions::default())
            .map_err(|e| eprintln!("Failed to start consumer: {}", e))
            .expect("Unable to create consumer!");

        // // Start a consumer.
        // let consumer = queue
        //     .consume(ConsumerOptions::default())
        //     c.expect("Failed to connect to create consumer");
        println!("Waiting for messages. Press Ctrl-C to exit.");

        for (i, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);
                    println!("({:>3}) Received [{}]", i, body);
                    consumer
                        .ack(delivery)
                        .expect("Failed to connect to RabbitMQ");
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }
        }
    }
}
