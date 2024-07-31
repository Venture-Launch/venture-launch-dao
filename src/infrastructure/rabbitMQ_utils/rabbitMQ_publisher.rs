use amqprs::callbacks::{DefaultChannelCallback, DefaultConnectionCallback};
use amqprs::channel::{BasicPublishArguments, Channel};
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::{BasicProperties, DELIVERY_MODE_PERSISTENT};

pub struct RabbitMQPublisher {
    connection: Connection,
    channel: Channel,
}

impl RabbitMQPublisher {
    pub async fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, String> {
        let connection_arguments = OpenConnectionArguments::new(host, port, username, password);

        let connection = Connection::open(&connection_arguments)
            .await
            .map_err(|e| e.to_string())?;

        connection
            .register_callback(DefaultConnectionCallback)
            .await
            .map_err(|e| e.to_string())?;

        let channel = connection.open_channel(None).await.map_err(|e| e.to_string())?;
        channel
            .register_callback(DefaultChannelCallback)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Self {
            connection,
            channel,
        })
    }

    pub async fn publish_message(&self, message: &str) -> Result<(), String> {
        if !self.connection.is_open() {
            return Err("Connection is not open".to_string());
        }

        if !self.channel.is_open() {
            return Err("Channel is not open".to_string());
        }

        let args = BasicPublishArguments::new("dao_exchange", "broker.response");

        self.channel
            .basic_publish(
                BasicProperties::default()
                    .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
                    .finish(),
                message.into(),
                args,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}