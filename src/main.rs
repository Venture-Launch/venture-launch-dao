pub mod infrastructure;
pub mod domain;

use dotenv::dotenv;
use std::error::Error;
use tokio;

use infrastructure::rabbitMQ_utils::broker::BrokerInitArgs;
use crate::infrastructure::rabbitMQ_utils::broker::Broker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let host = std::env::var("RABBIT_HOST").unwrap_or_else(|_| "localhost".into());

    let rabbit_broker = Broker::new(
            BrokerInitArgs {
                host,
                port: 5672,
                username: String::from("guest"),
                password: String::from("guest")
            }
        ).await.unwrap();

        let _ = tokio::join!(rabbit_broker.start_broker());

    Ok(())
}
