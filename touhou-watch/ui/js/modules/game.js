import { StageLocation } from './game_data/locations.js';
import SpellCard from './game_data/spell_card.js';
import ShotType from './game_data/shot_type.js';
import Difficulty from './game_data/difficulty.js';
import { BombEvent, BorderEndEvent, EndGameEvent, EnterSectionEvent, GameEvent, MissEvent, StartGameEvent } from './game_data/game_event.js';

export default class Game {
    /** @type {Date} */
    #startTime;

    /** @type {Date?} */
    #endTime = null;

    /** @type {StageLocation} */
    #currentLocation;

    /** @type {ShotType} */
    #shot;

    /** @type {boolean} */
    #practice;

    /** @type {Difficulty} */
    #difficulty;

    /** @type {StageLocation[]} */
    #misses = [];

    /** @type {StageLocation[]} */
    #bombs = [];

    /** @type {StageLocation[]} */
    #breaks = [];
    
    /** @type {StageLocation[]} */
    #locationsSeen = [];

    /** @type {boolean} */
    #cleared = false;

    /**
     * 
     * @param {StartGameEvent} startEvent 
     */
    constructor (startEvent) {
        this.#startTime = startEvent.time;
        this.#currentLocation = startEvent.location;
        this.#shot = startEvent.shot;
        this.#practice = startEvent.practice;
        this.#difficulty = startEvent.difficulty;

        this.#addSeenLocation(startEvent.location);
    }

    /** @returns {Date} */
    get startTime() {
        return this.#startTime;
    }

    /** @returns {Date?} */
    get endTime() {
        return this.#endTime;
    }
    
    /** @returns {StageLocation} */
    get currentLocation() {
        return this.#currentLocation;
    }

    /** @returns {Stage} */
    get currentStage() {
        return this.#currentLocation.stage;
    }

    /** @returns {ShotType} */
    get shot() {
        return this.#shot;
    }

    /** @returns {boolean} */
    get practice() {
        return this.#practice;
    }

    /** @returns {Difficulty} */
    get difficulty() {
        return this.#difficulty;
    }

    /** @returns {StageLocation[]} */
    get misses() {
        return this.#misses;
    }

    /** @returns {StageLocation[]} */
    get bombs() {
        return this.#bombs;
    }

    /** @returns {StageLocation[]} */
    get breaks() {
        return this.#breaks;
    }
    
    /** @returns {StageLocation[]} */
    get locationsSeen() {
        return this.#locationsSeen;
    }

    /** @returns {boolean} */
    get cleared() {
        return this.#cleared;
    }

    /** @returns {boolean} */
    get ended() {
        return !!this.#endTime;
    }

    /**
     * @param {StageLocation} location 
     */
    #addSeenLocation(location) {
        if (location.is_unknown || this.#locationsSeen.findIndex((elem) => elem.equals(location)) >= 0) return;
        this.#locationsSeen.push(location);
        this.#locationsSeen.sort((a, b) => a.compare(b));
    }

    forceEnd() {
        if (!this.ended) {
            this.endTime = new Date();
            this.cleared = false;
        }
    }

    /**
     * Update this game's state with a new event.
     * @param {GameEvent} event 
     */
    addEvent(event) {
        if (event instanceof EnterSectionEvent) {
            if (event.location.is_unknown && !this.#currentLocation.is_unknown) return;
            this.#addSeenLocation(event.location);
            this.#currentLocation = event.location;
        } else if (event instanceof EndGameEvent) {
            if (!event.location.is_unknown || this.#currentLocation.is_unknown) {
                this.#addSeenLocation(event.location);
                this.#currentLocation = event.location;
            }

            this.#endTime = event.time;
            this.#cleared = event.cleared;
        } else if (event instanceof MissEvent) {
            this.#addSeenLocation(event.location);
            this.#misses.push(event.location);
        } else if (event instanceof BombEvent) {
            this.#addSeenLocation(event.location);
            this.#bombs.push(event.location);
        } else if (event instanceof BorderEndEvent) {
            this.#addSeenLocation(event.location);
            if (event.broken) {
                this.#breaks.push(event.location);
            }
        }
    }

    /**
     * Returns an abbreviation listing the number of misses, bombs, and border breaks in this run.
     * @param {boolean} include_zeros Whether to include NM/NB/NBB in the resulting abbreviation. A perfect (0/0/0) run is always abbreviated as NNN.
     * @returns {string}
     */
    resultAbbreviation(include_zeros = false) {
        if (this.#misses.length === 0 && this.#bombs.length === 0 && this.#breaks.length === 0) {
            return "NNN";
        } else if (include_zeros) {
            return (
                (this.#misses.length === 0 ? "N" : this.#misses.length) + "M"
                + (this.#bombs.length === 0 ? "N" : this.#bombs.length) + "B"
                + (this.#breaks.length === 0 ? "N" : this.#breaks.length) + "BB"
            );
        } else {
            return (
                (this.#misses.length > 0 ? this.#misses.length + "M" : "")
                + (this.#bombs.length > 0 ? this.#bombs.length + "B" : "")
                + (this.#breaks.length > 0 ? this.#breaks.length + "BB" : "")
            );
        }
    }
}