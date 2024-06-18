use venture_launch_dao::amqp::consumer::Consumer;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    let rabbit_url = std::env::var("RABBIT_DEFAULT_URL").expect("RABBIT_DEFAULT_URL must be set.");

    let queue_name = std::env::var("QUEUE_NAME").expect("QUEUE_NAME must be set.");

    let consumer = Consumer::create(rabbit_url.as_str(), None, queue_name.as_str());
    consumer.init();
}
