# Etherface
This project, similar to [4Byte](https://github.com/pipermerriam/ethereum-function-signature-registry), aims to be an Ethereum Signature Database with two upgrades namely 
1) Etherface runs indefinitely finding signatures by itself, without any human intervention whatsoever. This is done by combination of crawling / polling websites where Solidity code can be found and consequently signatures can be scraped from (currently GitHub, Etherscan and 4Byte*). By comparison 4Byte relies on user submitted code, signatures and installed GitHub Webhooks.
2) Etherface provides source code references for its signatures, i.e. GitHub repositories / Etherscan URLs where these signatures were scraped from. The idea behind this is to give users a better understanding what a given signature might be used for.

Etherface is open-source and provides a frontend on [etherface.io](https://etherface.io/).

### Background
Function calls in the Ethereum network are specified by the first four byte of data sent with a transaction.
These first four bytes, also called function selector, represent the Keccak256 hash of the functions canonical form (e.g. `balanceOf(address)`).
As an outsider it is as such impossible to interpret what a given transaction does, because hashes are one-way
calculations. Furthermore [events](https://medium.com/mycrypto/understanding-event-logs-on-the-ethereum-blockchain-f4ae7ba50378)
and [errors](https://blog.soliditylang.org/2021/04/21/custom-errors/) are encoded in a similar fashion. Consequently rainbow tables are
needed to decode and inspect such signatures in the Ethereum network. While such rainbow tables exists,
most prominently [4Byte](https://www.4byte.directory/), two features are missing which Etherface tries to cover, as mentioned above.

## Architecture
Etherfaces architecture is kept simple for easier maintainability with three main modules, namely a `Fetcher`, `Scraper` and `REST API`. 
* The `Fetcher` modules is responsible for finding pages (URLs) with Solidity code / signatures from various websites by either crawling (GitHub) or polling (Etherscan, 4Byte) them. These pages along other metadata are then inserted into the database.
* The `Scraper` modules regularly retrieves the inserted data by the `Fetcher`, downloads these Solidity files (if present), extracts all signatures and inserts them into the database.
* The `REST API` is simply responsible for providing data to the outside world and is documented on [etherface.io/api-documentation](https://etherface.io/api-documentation)
<div align="center">
    <img src="https://github.com/volsa/etherface/blob/master/res/img/architecture_etherface.png?raw=true">
</div>

## Project Structure
* `etherface/` consists of the `Fetcher` and `Scraper` modules
* `etherface-lib` consists of library specific modules such as API clients, database handlers,...
* `etherface-rest` consits of the REST API
* `etherface-ui` consists of the NextJS webapp 

The project itself is heavily documented, which can be further inspected using Rustdoc (`cargo doc --open --no-deps`)


## Installation (Development)
**Note:** This will at some point be dockerized for easier deployment / contribution, for now the following steps should however give you both a development and production enviroment. 
1. Install
    * Rust (https://www.rust-lang.org/tools/install)
    * Docker (https://docs.docker.com/engine/install/ubuntu/)
    * Etherface dependencies
        ```
        sudo apt install build-essential docker-compose pkg-config libpq-dev libssl-dev

        # You might need to start another bash instance if Rust has been freshly installed
        cargo install diesel_cli --no-default-features --features postgres
        ```
2. Populate the `.env` file
3. Execute `docker-compose up -d`
4. Log into postgres, create the `etherface` database and run the diesel-rs migration
    ```
    # Create etherface database
    docker ps                           # copy the CONTAINER ID of the IMAGE 'postgres:XX'
    docker exec -it <CONTAINER_ID> bash

    psql -U root
    CREATE DATABASE etherface;          # Inside the psql CLI

    # Run the diesel-rs migration
    diesel migration run
    ```
6. Start `etherface`, `etherface-rest` or `etherface-ui`, best done within a tmux session
    ```
    # In the ./etherface folder
    cargo r --release --bin etherface
    cargo r --release --bin etherface-rest

    # In the ./etherface/etherface-ui folder
    npm run dev
    ```
