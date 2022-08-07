# Installation
1. Install
    * Rust (https://www.rust-lang.org/tools/install)
    * Docker (https://docs.docker.com/engine/install/ubuntu/)
    * Etherface dependencies
        ```
        sudo apt install build-essential docker-compose pkg-config libpq-dev libssl-dev

        # You might need to start another bash instance if Rust has been freshly installed
        cargo install diesel_cli --no-default-features --features postgres
        ```
2. Rename the `etherface-EXAMPLE.toml` file to `etherface.toml` and configure it
3. Set and export the  `DATABASE_PASSWORD` environment variable, like so:
    ```
    export DATABASE_PASSWORD='root' # Don't actually use root if the database is publicy accessible
    echo $DATABASE_PASSWORD
    ```
4. Execute `docker-compose up -d`
5. Log into postgres, create the `etherface` database and run the diesel-rs migration
    ```
    # Create etherface database
    docker ps                           # copy the CONTAINER ID of the IMAGE 'postgres:XX'
    docker exec -it <CONTAINER_ID> bash

    psql -U root
    create database etherface;          # Inside the psql CLI

    # Run the diesel-rs migration
    export DATABASE_URL=postgres://root:<DATABASE_PASSWORD>@localhost/etherface
    diesel migration run
    ```
6. Start `etherface`, `etherface-rest` or `etherface-ui`, best done within a tmux session
    ```
    # In the etherface folder
    cargo r --release --bin etherface
    cargo r --release --bin etherface-rest

    # In the etherface-ui folder
    yarn start
    ```