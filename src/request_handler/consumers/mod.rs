mod create_dao;
mod add_member;
mod remove_member;
mod change_threshold;
mod vote;
mod withdraw;
mod execute_proposal;

use amqp_serde::types::{FieldName, FieldValue};
use amqprs::channel::{BasicAckArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use async_trait::async_trait;
use change_threshold::ChangeThresholdDaoSchema;
use execute_proposal::ProposalExecuteDaoSchema;
use serde::Deserialize;
use serde_json;
use vote::VoteDaoSchema;
use withdraw::WithdrawDaoSchema;

use crate::request_handler::consumers::create_dao::CreateDaoSchema;
use crate::request_handler::consumers::add_member::AddMemberDaoSchema;
use crate::request_handler::consumers::remove_member::RemoveMemberDaoSchema;

use super::processor::RabbitMQPublisher;

pub struct RabbitMQConsumer {
}

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

// mod create_dao;
// mod add_member;
// mod remove_member;
// mod change_threshold;
// mod vote;
// mod withdraw;
// mod execute_proposal;

async fn run_consumer(consumer_name: &str, raw_json_schema: &str) -> Result<String, String> {
    return match consumer_name {
        "create_dao" => {
            println!("creating dao");
            let json: CreateDaoSchema = load_schema(raw_json_schema)?;
            println!("{:?}",json);
            create_dao::consume(json).await
        },
        "add_member" => {
            let json: AddMemberDaoSchema = load_schema(raw_json_schema)?;
            println!("{:?}",json);
            add_member::consume(json).await
        },
        "remove_member" => {
            let json: RemoveMemberDaoSchema = load_schema(raw_json_schema)?;
            println!("{:?}",json);
            remove_member::consume(json).await
        },
        "change_threshold" => {
            let json: ChangeThresholdDaoSchema = load_schema(raw_json_schema)?;
            println!("{:?}",json);
            change_threshold::consume(json).await
        },
        "vote" => {
            let json: VoteDaoSchema = load_schema(raw_json_schema)?;
            println!("{:?}",json);
            vote::consume(json).await
        },
        "withdraw" => {
            let json: WithdrawDaoSchema = load_schema(raw_json_schema)?;
            println!("{:?}",json);
            withdraw::consume(json).await
        },
        "execute_proposal" => {
            let json: ProposalExecuteDaoSchema = load_schema(raw_json_schema)?;
            println!("{:?}",json);
            execute_proposal::consume(json).await
        },
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

        let publisher = RabbitMQPublisher::new("localhost", 5672, "guest", "guest").await.unwrap();

        let result: Result<String, String> = run_consumer(&command, raw_string).await;

        match result {
            Ok(success_message) => {
                let _ = publisher.publish_message(&format!("{{\"msg\": \"{success_message}\"}}")).await;
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
