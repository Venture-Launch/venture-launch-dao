use amqprs::{channel::{BasicAckArguments, Channel}, consumer::AsyncConsumer, BasicProperties, Deliver, FieldName, FieldValue};
use async_trait::async_trait;
use serde::Deserialize;
use crate::infrastructure::rabbitMQ_utils::rabbitMQ_publisher::RabbitMQPublisher;
use crate::infrastructure::request_handler::consumers::{add_member::{self, AddMemberDaoSchema}, change_threshold::{self, ChangeThresholdDaoSchema}, create_dao::{self, CreateDaoSchema}, execute_proposal::{self, ProposalExecuteDaoSchema}, remove_member::{self, RemoveMemberDaoSchema}, vote::{self, VoteDaoSchema}, withdraw::{self, WithdrawDaoSchema}};
pub struct RabbitMQConsumer {
    publisher: Option<RabbitMQPublisher>
}

impl RabbitMQConsumer {
    pub fn new() -> RabbitMQConsumer {
        return RabbitMQConsumer {
            publisher: None
        };
    }

    pub async fn init_publisher(
        &mut self,
        host: &str,
        port: u16,
        username: &str,
        password: &str,) {
        let publisher = RabbitMQPublisher::new(host, port, username, password).await.unwrap();

        self.publisher = Some(publisher);
    }

    fn load_schema<'a, T: Deserialize<'a>>(&self, raw_json: &'a str) -> Result<T, String> {
        return match serde_json::from_str::<T>(raw_json) {
            Ok(json_schema) => Ok(json_schema),
            Err(..) => Err("Could not parse raw string into json".to_string()),
        };
    }

    async fn run_consumer(&self, consumer_name: &str, raw_json_schema: &str) -> Result<String, String> {
        return match consumer_name {
            "create_dao" => {
                println!("creating dao");
                let json: CreateDaoSchema = self.load_schema(raw_json_schema)?;
                println!("{:?}",json);
                create_dao::consume(json).await
            },
            "add_member" => {
                let json: AddMemberDaoSchema = self.load_schema(raw_json_schema)?;
                println!("{:?}",json);
                add_member::consume(json).await
            },
            "remove_member" => {
                let json: RemoveMemberDaoSchema = self.load_schema(raw_json_schema)?;
                println!("{:?}",json);
                remove_member::consume(json).await
            },
            "change_threshold" => {
                let json: ChangeThresholdDaoSchema = self.load_schema(raw_json_schema)?;
                println!("{:?}",json);
                change_threshold::consume(json).await
            },
            "vote" => {
                let json: VoteDaoSchema = self.load_schema(raw_json_schema)?;
                println!("{:?}",json);
                vote::consume(json).await
            },
            "withdraw" => {
                let json: WithdrawDaoSchema = self.load_schema(raw_json_schema)?;
                println!("{:?}",json);
                withdraw::consume(json).await
            },
            "execute_proposal" => {
                let json: ProposalExecuteDaoSchema = self.load_schema(raw_json_schema)?;
                println!("{:?}",json);
                execute_proposal::consume(json).await
            },
            unknown_command => Err(format!("Unknown command: {}", unknown_command)),
        };
    }
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

        let result: Result<String, String> = self.run_consumer(&command, raw_string).await;

        match result {
            Ok(success_message) => {
                if let Some(publisher) = &self.publisher {
                    let _ = publisher.publish_message(format!("{{\"command_type\" : \"{command}\", {success_message}}}").as_str()).await;
                }
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
