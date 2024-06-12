# DAO service for VentureLaunch

Run RabbitMQ:

```bash
docker run -d --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:3-management
```

Run service:

```bash
cargo run
```

Default `.env` file (must be in root directory):
| Name | Value |
| :---: | :---: |
| QUEUE_NAME | queue |
| RABBIT_DEFAULT_URL | amqp://localhost |
