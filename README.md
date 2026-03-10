# Scenario-Queue Manager

This is a task manager for the PISA Scenario-Queue project. It is responsible for managing the tasks that are created by the users and assigning them to the appropriate executors.

## Usage

### Configuration

Copy the `.env.example` file to `.env` and fill in the required environment variables.

### Running the Application

To run the scenario-queue manager, you need to have a postgreSQL database set up.
You can use the provided `docker-compose.yml` file to set up the database and the manager itself.

```bash
docker-compose up -f docker/docker-compose.yml --env-file .env
```

To start the manager, run the following command:

```bash
cargo build --release
./target/release/manager
```

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue if you have any ideas or suggestions.
