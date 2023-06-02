CREATE TABLE IF NOT EXISTS spellcards
(
    ts        DATETIME NOT NULL,
    card_id   INTEGER NOT NULL,
    shot_type INTEGER NOT NULL,
    captures  INTEGER NOT NULL,
    attempts  INTEGER NOT NULL,
    max_bonus INTEGER NOT NULL,
    PRIMARY KEY (ts, card_id, shot_type)
);
