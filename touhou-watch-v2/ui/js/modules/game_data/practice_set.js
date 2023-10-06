import { NamedValue, GameId } from "./named_value.js";
import GameTime from "./game_time.js";

export class GameLocation  {
    /** @type {number} */
    #value;

    /** @type {string} */
    #name;

    /** @type {NamedValue} */
    #stage;
    
    /** @type {NamedValue?} */
    #spell;

    /**
     * 
     * @param {number} value
     * @param {string} name 
     * @param {NamedValue} stage 
     * @param {NamedValue?} spell 
     */
    constructor (value, name, stage, spell) {
        this.#value = value;
        this.#name = name;
        this.#stage = stage;
        this.#spell = spell;
    }

    /**
     * Deserialize a GameLocation directly from Rust data.
     * 
     * @param {Object} src 
     * @returns {GameLocation}
     */
    static deserialize(src) {
        return new GameLocation(
            src.value,
            src.name,
            NamedValue.deserialize(src.stage),
            (src.spell != null) ? NamedValue.deserialize(src.spell) : null
        );
    }

    /** @returns {number} */
    get value() {
        return this.#value;
    }

    /** @returns {string} */
    get name() {
        if (this.#spell) {
            return this.#spell.name;
        } else {
            return this.#stage.name + " " + this.#name;
        }
    }

    /** @type {NamedValue} */
    get stage() {
        return this.#stage;
    }

    /** @type {NamedValue?} */
    get spell() {
        return this.#spell;
    }

    /**
     * 
     * @param {GameLocation} other 
     * @returns {boolean}
     */
    compare(other) {
        if (!(other instanceof GameLocation)) {
            return 1;
        }

        return this.#value - other.#value;
    }

    valueOf() {
        return this.#value;
    }

    toString() {
        return this.#name;
    }
    
    [Symbol.toPrimitive](hint) {
        if (hint === 'number') {
            return this.#value;
        } else {
            return this.name;
        }
    }
}

export class Attempt {
    /** @type {GameTime} */
    #startTime;

    /** @type {GameTime} */
    #endTime;

    /** @type {boolean} */
    #success;

    constructor (startTime, endTime, success) {
        this.#startTime = startTime;
        this.#endTime = endTime;
        this.#success = success;
    }

    /**
     * Deserialize an Attempt directly from Rust data.
     * 
     * @param {Object} src 
     * @returns {Attempt}
     */
    static deserialize(src) {
        return new Attempt(
            GameTime.deserialize(src.start_time),
            GameTime.deserialize(src.end_time),
            src.success
        );
    }

    /** @returns {GameTime} */
    get startTime() {
        return this.#startTime;
    }

    /** @returns {GameTime} */
    get endTime() {
        return this.#endTime;
    }

    /** @returns {number} */
    get duration() {
        return this.#endTime.realTimeBetween(this.#startTime);
    }

    /** @returns {number} */
    get playTime() {
        return this.#endTime.playTimeBetween(this.#startTime);
    }

    /** @returns {boolean} */
    get success() {
        return this.#success;
    }

    /**
     * 
     * @param {Attempt} other 
     * @returns {boolean}
     */
    compare(other) {
        if (!(other instanceof Attempt)) {
            return 1;
        }

        return this.#startTime.valueOf() - other.#startTime.valueOf();
    }

    valueOf() {
        return this.#startTime.valueOf();
    }
    
    [Symbol.toPrimitive](hint) {
        return this.#startTime.valueOf();
    }
}

export class SetMetrics {
    static #INTERNAL = false;

    /** @type {number} */
    #attempts;

    /** @type {number} */
    #captures;

    /** @type {number} */
    #totalTime;

    constructor (attempts, captures, totalTime) {
        if (!SetMetrics.#INTERNAL) {
            throw new TypeError("SetMetrics cannot be instantiated directly");
        }
        SetMetrics.#INTERNAL = false;

        this.#attempts = attempts;
        this.#captures = captures;
        this.#totalTime = totalTime;
    }

    /**
     * 
     * @param {Attempt[]} attempts 
     */
    static fromAttempts(attempts) {
        SetMetrics.#INTERNAL = true;
        return new SetMetrics(
            attempts.length,
            attempts.reduce((acc, attempt) => acc + (attempt.success ? 1 : 0), 0),
            attempts.reduce((acc, attempt) => acc + attempt.playTime, 0)
        );
    }

    /**
     * 
     * @param {SetMetrics[]} sets 
     * @returns {SetMetrics}
     */
    static total(sets) {
        var totals = sets.reduce((acc, val) => {
            acc.attempts += val.#attempts;
            acc.captures += val.#captures;
            acc.time += val.#totalTime;
            return acc;
        }, { attempts: 0, captures: 0, time: 0 });

        SetMetrics.#INTERNAL = true;
        return new SetMetrics(totals.attempts, totals.captures, totals.time);
    }

    /** @returns {number} */
    get attempts() {
        return this.#attempts;
    }

    /** @returns {number} */
    get captures() {
        return this.#captures;
    }

    /** @returns {number} */
    get totalTime() {
        return this.#totalTime;
    }

    /** @returns {number} */
    get capRate() {
        return this.captures / this.attempts;
    }

    /** @returns {number} */
    get averageTime() {
        return this.totalTime / this.attempts;
    }

    /**
     * 
     * @param {SetMetrics} other 
     * @returns {SetMetrics}
     */
    add(other) {
        SetMetrics.#INTERNAL = true;
        return new SetMetrics(
            this.#attempts + other.#attempts,
            this.#captures + other.#captures,
            this.#totalTime + other.#totalTime
        );
    }
}

export class PracticeAttempts {
    /** @type {GameId} */
    #game;

    /** @type {NamedValue} */
    #shotType;

    /** @type {NamedValue} */
    #difficulty;

    /** @type {GameLocation} */
    #location;

    /** @type {Attempt[]} */
    #attempts;

    /**
     * 
     * @param {GameId} game 
     * @param {NamedValue} shotType 
     * @param {NamedValue} difficulty 
     * @param {GameLocation} location 
     * @param {Attempt[]} attempts 
     */
    constructor (game, shotType, difficulty, location, attempts) {
        this.#game = game;
        this.#shotType = shotType;
        this.#difficulty = difficulty;
        this.#location = location;
        this.#attempts = attempts;
    }
    
    /**
     * Deserialize PracticeAttempts directly from Rust data.
     * 
     * @param {Object} src 
     * @returns {PracticeAttempts}
     */
    static deserialize(src) {
        return new PracticeAttempts(
            GameId.deserialize(src.game),
            NamedValue.deserialize(src.shot_type),
            NamedValue.deserialize(src.difficulty),
            GameLocation.deserialize(src.location),
            src.attempts.map(Attempt.deserialize)
        );
    }

    /** @returns {GameId} */
    get game() {
        return this.#game;
    }

    /** @returns {NamedValue} */
    get shotType() {
        return this.#shotType;
    }

    /** @returns {NamedValue} */
    get difficulty() {
        return this.#difficulty;
    }

    /** @returns {GameLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {string} */
    get key() {
        return this.#game.value + ":" + this.#shotType.value + ":" + this.#difficulty.value + ":" + this.#location.value;
    }

    /** @returns {Attempt[]} */
    attempts() {
        return this.#attempts;
    }

    /** @returns {GameTime} */
    lastAtempt() {
        if (this.#attempts.length == 0) return new Date(0);
        return this.#attempts.reduce(
            (acc, val) => (val.startTime.timestamp.valueOf() > acc.valueOf()) ? val : acc, 
            this.#attempts[0].startTime.timestamp
        );
    }

    /** @returns {Attempt[][]} */
    sets() {
        return this.#attempts.reduce((acc, val) => {
            if (acc[acc.length - 1].length >= 25) {
                acc.push([]);
            }

            acc[acc.length - 1].push(val);
            return acc;
        }, [[]]);
    }

    /** @returns {SetMetrics[]} */
    setMetrics() {
        return this.sets().map(SetMetrics.fromAttempts);
    }

    /** @returns {SetMetrics} */
    totalMetrics() {
        return SetMetrics.fromAttempts(this.#attempts);
    }
}
