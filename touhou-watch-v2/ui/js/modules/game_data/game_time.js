import { formatDuration } from "../utils.js";

export default class GameTime {
    /** @type {Date} */
    #timestamp;

    /** @type {number} */
    #relativeReal;

    /** @type {number} */
    #relativeGame;

    /**
     * 
     * @param {Date} timestamp 
     * @param {number} relativeReal 
     * @param {number} relativeGame 
     */
    constructor (timestamp, relativeReal, relativeGame) {
        this.#timestamp = timestamp;
        this.#relativeReal = relativeReal;
        this.#relativeGame = relativeGame;
    }

    /**
     * 
     * @param {Object} src 
     * @returns {GameTime}
     */
    static deserialize(src) {
        return new GameTime(
            new Date(src.timestamp),
            src.relative_real_time,
            src.relative_game_time,
        );
    }

    /** @returns {Date} */
    get timestamp() {
        return this.#timestamp
    }

    /** @returns {number} */
    get timestampMs() {
        return this.#timestamp.valueOf()
    }

    /** @returns {number} */
    get relativeRealTime() {
        return this.#relativeReal
    }

    /** @returns {number} */
    get relativeGameTime() {
        return this.#relativeGame
    }

    /**
     * 
     * @param {GameTime} other 
     * @returns {number}
     */
    realTimeBetween(other) {
        var a = this.#timestamp.valueOf();
        var b = other.#timestamp.valueOf();
        if (a <= b) {
            return b - a;
        } else {
            return a - b;
        }
    }

    /**
     * 
     * @param {GameTime} other 
     * @returns {number}
     */
    playTimeBetween(other) {
        var a = this.#relativeGame;
        var b = other.#relativeGame;
        if (a <= b) {
            return b - a;
        } else {
            return a - b;
        }
    }

    /**
     * 
     * @param {number?} precision 
     * @returns {string}
     */
    formatRealTime(precision = 1) {
        return formatDuration(this.#relativeReal, precision);
    }

    /**
     * 
     * @param {number?} precision 
     * @returns {string}
     */
    formatGameTime(precision = 1) {
        return formatDuration(this.#relativeGame, precision);
    }

    valueOf() {
        return this.#timestamp.valueOf();
    }
    
    [Symbol.toPrimitive](hint) {
        return this.#timestamp.valueOf();
    }
}