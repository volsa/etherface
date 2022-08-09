CREATE TYPE signature_kind          AS ENUM ('function', 'event', 'error');

CREATE TABLE github_crawler_metadata (
    id                              INT                         NOT NULL,
    last_user_check                 TIMESTAMP WITH TIME ZONE    NOT NULL,
    last_repository_check           TIMESTAMP WITH TIME ZONE    NOT NULL,
    last_repository_search          TIMESTAMP WITH TIME ZONE    NOT NULL,

    PRIMARY KEY (id)
);
INSERT INTO github_crawler_metadata VALUES (1, NOW(), NOW(), NOW());

CREATE TABLE github_user (
    id                                      INT                         NOT NULL,
    login                                   TEXT                        NOT NULL,
    html_url                                TEXT                        NOT NULL,
    is_deleted                              BOOLEAN                     NOT NULL,

    -- The following fields are not part of the official API response
    added_at                                TIMESTAMP WITH TIME ZONE    NOT NULL,
    visited_at                              TIMESTAMP WITH TIME ZONE,

    PRIMARY KEY (id)
);

CREATE TABLE github_repository (
    id                  INT                         NOT NULL,
    owner_id            INT                         NOT NULL,
    name                TEXT                        NOT NULL,
    html_url            TEXT                        NOT NULL,
    language            TEXT,
    stargazers_count    INT                         NOT NULL,
    size                INT                         NOT NULL,
    fork                BOOLEAN                     NOT NULL,
    fork_parent_id      INT,
    created_at          TIMESTAMP WITH TIME ZONE    NOT NULL,
    pushed_at           TIMESTAMP WITH TIME ZONE    NOT NULL,
    updated_at          TIMESTAMP WITH TIME ZONE    NOT NULL,


    -- The following fields are not part of the official API response
    scraped_at          TIMESTAMP WITH TIME ZONE,               -- date we last scraped signatures from the repository
    visited_at          TIMESTAMP WITH TIME ZONE,               -- date we last visited the repository
    added_at            TIMESTAMP WITH TIME ZONE    NOT NULL,   -- date we added the repository into the database

    solidity_ratio      REAL,
    is_deleted          BOOLEAN                     NOT NULL, -- flag indicating if repository is deleted 
    found_by_crawling   BOOLEAN                     NOT NULL, -- flag indicating if repository was found using the API search endpoint or by crawling

    PRIMARY KEY (id),
    FOREIGN KEY (owner_id)          REFERENCES github_user (id),
    FOREIGN KEY (fork_parent_id)    REFERENCES github_repository (id)
);

CREATE TABLE etherscan_contract (
    id                  SERIAL                      NOT NULL,
    address             TEXT                        NOT NULL,
    name                TEXT                        NOT NULL,
    compiler            TEXT                        NOT NULL,
    compiler_version    TEXT                        NOT NULL,
    url                 TEXT                        NOT NULL,
    scraped_at          TIMESTAMP WITH TIME ZONE,

    -- The following fields are not part of the official API response
    added_at            TIMESTAMP WITH TIME ZONE    NOT NULL,

    UNIQUE (address),
    PRIMARY KEY (id)
);

CREATE TABLE signature (
    id          SERIAL                      NOT NULL,
    text        TEXT                        NOT NULL,
    hash        TEXT                        NOT NULL,
    added_at    TIMESTAMP WITH TIME ZONE    NOT NULL,

    UNIQUE (hash),
    PRIMARY KEY (id)
);

CREATE TABLE mapping_signature_github (
    signature_id    INT                         NOT NULL REFERENCES signature           (id),
    repository_id   INT                         NOT NULL REFERENCES github_repository   (id),
    kind            SIGNATURE_KIND              NOT NULL, 
    added_at        TIMESTAMP WITH TIME ZONE    NOT NULL,

    PRIMARY KEY (signature_id, repository_id, kind)
);

CREATE TABLE mapping_signature_etherscan (
    signature_id    INT                         NOT NULL REFERENCES signature            (id),
    contract_id     INT                         NOT NULL REFERENCES etherscan_contract   (id),
    kind            SIGNATURE_KIND              NOT NULL,
    added_at        TIMESTAMP WITH TIME ZONE    NOT NULL,

    PRIMARY KEY (signature_id, contract_id, kind)
);

CREATE TABLE mapping_signature_fourbyte (
    signature_id    INT                         NOT NULL REFERENCES signature            (id),
    kind            SIGNATURE_KIND              NOT NULL,
    added_at        TIMESTAMP WITH TIME ZONE    NOT NULL,

    PRIMARY KEY (signature_id, kind)
);