**This project has been sunset, all its data has been transferred over to [sourcify](https://4byte.sourcify.dev/).**

What started as a bachelor's thesis project to get a better understanding of Rust and the Ethereum blockchain has become, objectively speaking, a better alternative to [4Byte](https://www.4byte.directory/). With over 4.7 million signatures, Etherface at some point represented (to my knowledge) the biggest publicly available database of Ethereum signatures. After three years, I have decided to sunset the project due to lack of time and interest. All data has been transferred over to [sourcify](https://4byte.sourcify.dev/).

# Etherface
Similar to [4Byte](https://www.4byte.directory/) and [eth.samczsun](https://sig.eth.samczsun.com/), Etherface is an Ethereum Signature Database hosting over 4.7 million function, event and error signatures, making it the biggest publicly available database. Unlike other providers that rely primarily on user submissions, Etherface utilizes a specialized crawler that runs 24/7 to autonomously discover signatures from GitHub, Etherscan, and 4Byte. This automated approach eliminates the need for manual curation and ensures continuous growth.

Additionally, this crawling architecture allows Etherface to provide context for every signature. A search result returns not just the decoded clear-text signature, but also its source, such as the specific GitHub repository or Etherscan address where it was discovered. [Try it out](https://www.etherface.io/hash).

## Signature Databases
Every transaction in Ethereum can carry additional input data, for example take the following [arbitrary transaction](https://etherscan.io/tx/0x2b930225479934eda949c3c2b0f3af5d5fd60136f7c9f0d5bbabf569def1f8a8) found on Etherscan.
Its input data consists of  `0x095ea7b...85190000` and at first sight might look uninteresting. 
This specific piece of data however is an encoded [Keccak256](https://emn178.github.io/online-tools/keccak_256.html) hash and is essential when communicating with smart contracts. 
Specifically the first 4 bytes of this input data, i.e. `0x095ea7b3`, specify which function in the smart contract gets executed. 
However, since hashes are not reversible we cannot simply decode `0x095ea7b3` back into its original function name. Instead, we need a rainbow table of precomputed signatures to match the hash. This is the core purpose of Etherface: it maps these opaque 4-byte selectors back to human-readable text, bringing transparency to Ethereum transactions.
For example `0x095ea7b3` has a mapping to `approve(address,uint256)`, thus we know there's a transaction between A and B interacting with the `approve` function.

## Architecture
Etherface's architecture consists of three main modules, namely the `Fetcher`, `Scraper` and `REST API`.  
* The `Fetcher` module is responsible for finding pages (URLs) with Solidity code / signatures from various websites by either crawling (GitHub) or polling (Etherscan, 4Byte) them. These pages along other metadata are then inserted into the database.
* The `Scraper` module regularly retrieves the inserted data by the `Fetcher`, downloads these Solidity files (if present), extracts all signatures and inserts them into the database.
* The `REST API` is responsible for providing data to the outside world and is documented on [etherface.io/api-documentation](https://etherface.io/api-documentation)
<div align="center">
    <img src="https://github.com/volsa/etherface/blob/master/res/img/architecture_etherface.png?raw=true">
</div>

## Project Structure
* `etherface/`: Contains the `Fetcher` and `Scraper` modules.
* `etherface-lib`: Holds shared library modules (API clients, database handlers, etc.).
* `etherface-rest`: Implements the REST API.
* `etherface-ui`: The NextJS web application. 

The project itself is heavily documented, which can be further inspected using Rustdoc via `cargo doc --open --no-deps`


## Installation (Development)
1. Install
    * [Rust](https://www.rust-lang.org/tools/install)
    * [Docker](https://docs.docker.com/engine/install/ubuntu/)
    * Etherface dependencies
        ```
        sudo apt install build-essential docker-compose pkg-config libpq-dev libssl-dev

        # You might need to start another bash instance if Rust has been freshly installed
        cargo install diesel_cli --no-default-features --features postgres
        ```
2. Populate the `.env` file
    * GitHub tokens to be used are "Classic" and of the form `ghp_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`. The token will work with the default permissions - nothing additional must be granted.
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
5. Start `etherface`, `etherface-rest` or `etherface-ui`, best done within a tmux session
    ```
    # In the ./etherface folder
    cargo r --release --bin etherface
    cargo r --release --bin etherface-rest

    # In the ./etherface/etherface-ui folder
    npm install
    npm run dev
    ```

## Acknowledgements
Etherface was started as the project for my bachelor's thesis. I would like to thank the [Security and Privacy Lab](https://informationsecurity.uibk.ac.at/) at the University of Innsbruck and my [supervisor](https://informationsecurity.uibk.ac.at/people/michael-froewis/) for the support and inspiration.
