pub struct Controller;

impl Controller {
    pub async fn handle_message(message: String) {
        let service = BusinessService::new();
        service.process_data(message).await;
    }
}
