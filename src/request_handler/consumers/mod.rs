mod create_user;
mod delete_user;

use amqp_serde::types::{FieldName, FieldValue};
use amqprs::channel::{BasicAckArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json;

use crate::request_handler::consumers::create_user::CreateUserSchema;
use crate::request_handler::consumers::delete_user::DeleteUserSchema;

pub struct RabbitMQConsumer {}

impl RabbitMQConsumer {
    pub fn new() -> RabbitMQConsumer {
        return RabbitMQConsumer {};
    }
}

fn load_schema<'a, T: Deserialize<'a>>(raw_json: &'a str) -> Result<T, String> {
    return match serde_json::from_str::<T>(raw_json) {
        Ok(json_schema) => Ok(json_schema),
        Err(..) => Err("Could not parse raw string into json".to_string()),
    };
}

async fn run_consumer(consumer_name: &str, raw_json_schema: &str) -> Result<String, String> {
    return match consumer_name {
        "create_user" => {
            let json: CreateUserSchema = load_schema(raw_json_schema)?;

            create_user::consume(json).await
        }
        "delete_user" => {
            let json: DeleteUserSchema = load_schema(raw_json_schema)?;

            delete_user::consume(json).await
        }
        unknown_command => Err(format!("Unknown command: {}", unknown_command)),
    };
}

#[async_trait]
impl AsyncConsumer for RabbitMQConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .expect("Could not send acknowledgement!");

        let raw_string = match std::str::from_utf8(&content) {
            Ok(raw_string) => raw_string,
            Err(..) => {
                println!("Could not parse byte content into raw string");
                return;
            }
        };

        let command_header_key: FieldName = "command".try_into().unwrap();
        let headers = match basic_properties.headers() {
            Some(headers) => headers,
            None => {
                println!("Headers was not provided");
                return;
            }
        };
        let command = match headers.get(&command_header_key) {
            Some(command) => command,
            None => {
                println!("'command' header was not provided");
                return;
            }
        };

        let command = match command {
            FieldValue::S(command) => command.to_string(),
            _ => {
                println!("'command' header must be a string");
                return;
            }
        };

        let result: Result<String, String> = run_consumer(&command, raw_string).await;

        match result {
            Ok(success_message) => {
                println!(
                    "[{:?} RABBITMQ INFO] {}",
                    chrono::Utc::now(),
                    success_message
                );
            }
            Err(error_message) => {
                eprintln!(
                    "[{:?} RABBITMQ ERROR] {}",
                    chrono::Utc::now(),
                    error_message
                );
            }
        }
    }
}
