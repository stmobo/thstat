CREATE TABLE IF NOT EXISTS practices
(
    ts         DATETIME NOT NULL,
    difficulty INTEGER NOT NULL,
    shot_type  INTEGER NOT NULL,
    stage      INTEGER NOT NULL,
    attempts   INTEGER NOT NULL,
    high_score INTEGER NOT NULL,
    PRIMARY KEY (ts, difficulty, shot_type, stage)
);
