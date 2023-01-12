# Etherface
Similar to [4Byte](https://www.4byte.directory/) and [eth.samczsun](https://sig.eth.samczsun.com/), Etherface is an Ethereum Signature Database currently hosting ~2.25 million function, event and error signatures making it the biggest publicly available database. This is made possible because Etherface compared to other providers consists of a crawler, finding signatures from GitHub, Etherscan and 4Byte. Consequentely this means Etherface runs 24/7 finding new signatures indefinitely without any human-based guiding. For comparison, 4Byte relies on user submitted data whereas eth.samczsun also relies on user submitted data but was (probably) initially seeded with some private database.

Furthermore because a crawler is implemented, Etherface provides not only signatures but also where these signatures where found at. That is if you search for a signature on Etherface, you'll most likely get a response with it's decoded clear-text form as well as either a GitHub repository or a Etherscan address. [Try it out](https://www.etherface.io/hash).

## What's a signature database?
Every transaction in Ethereum can carry additional input data, for example take the following [arbitrary transaction](https://etherscan.io/tx/0x2b930225479934eda949c3c2b0f3af5d5fd60136f7c9f0d5bbabf569def1f8a8) found on Etherscan.
Its input data consists of  `0x095ea7b...85190000` and at first sight might look uninteresting. 
This specific piece of data however is encoded [Keccak256](https://emn178.github.io/online-tools/keccak_256.html) hash and is essential when communicating with smart contracts. 
Specifically the first 4 byte of this input data, i.e. `0x095ea7b3`, specify which function in the smart contract gets executed. 
What exactly does `0x095ea7b3` translate to though, clearly some mapping between hashes and their clear-text form is needed right? 
This is where signature databases come into play, making transactions in the network more transparent.
For example `0x095ea7b3` has a mapping to `approve(address,uint256)`, thus we know there's a transaction between A and B interacting with the `approve` function.

## Acknowlegements
Etherface was started as the project for my bachelors thesis. I would like to thank the [Security and Privacy Lab](https://informationsecurity.uibk.ac.at/) at the University of Innsbruck and my [supervisor](https://informationsecurity.uibk.ac.at/people/michael-froewis/') for the support and inspiration.

## Cite
...

# Development
Etherface _might_ undergo a huge refactor, making the architecture async by nature and even more extensible for future website to crawl from.
For now though the following sections describe it's current architecture and deployment instructions.
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

The project itself is heavily documented, which can be further inspected using Rustdoc via `cargo doc --open --no-deps`


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
    * Github tokens to be used are "Classic" and of the form `ghp_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`. The token will work with the default permissions - nothing additional must be granted.
4. Execute `docker-compose up -d`
5. Log into postgres, create the `etherface` database and run the diesel-rs migration
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
    npm install
    npm run dev
    ```
