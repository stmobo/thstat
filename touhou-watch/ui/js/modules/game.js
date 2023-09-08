import { StageLocation } from './game_data/locations.js';
import SpellCard from './game_data/spell_card.js';
import ShotType from './game_data/shot_type.js';
import Difficulty from './game_data/difficulty.js';
import { BombEvent, BorderEndEvent, EndGameEvent, EnterSectionEvent, GameEvent, MissEvent, StartGameEvent } from './game_data/game_event.js';

export default class Game {
    static #internalOnly = false;

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

    /** @type {[Date, StageLocation][]} */
    #misses = [];

    /** @type {[Date, StageLocation][]} */
    #bombs = [];

    /** @type {[Date, StageLocation][]} */
    #breaks = [];
    
    /** @type {StageLocation[]} */
    #locationsSeen = [];

    /** @type {boolean} */
    #cleared = false;

    /** @type {number} */
    #score;

    /** @type {number} */
    #continues;

    /** @type {GameEvent[]} */
    #events = [];

    /**
     * 
     * @param {Date} startTime 
     * @param {StageLocation} startLocation 
     * @param {ShotType} shot 
     * @param {boolean} practice 
     * @param {Difficulty} difficulty 
     */
    constructor (startTime, startLocation, shot, practice, difficulty) {
        if (!Game.#internalOnly) {
            throw new TypeError("Game instances cannot be constructed directly, use Game.deserialize or Game.fromStartEvent instead");
        }
        Game.#internalOnly = false;

        this.#startTime = startTime;
        this.#currentLocation = startLocation;
        this.#shot = shot;
        this.#practice = practice;
        this.#difficulty = difficulty;

        this.#addSeenLocation(startLocation);
    }

    /**
     * Deserialize a Game from raw event data.
     * 
     * @param {*} src 
     * @returns {Game}
     */
    static deserialize(src) {
        Game.#internalOnly = true;
        var ret = new Game(
            new Date(src.start_time),
            StageLocation.deserialize(src.location),
            ShotType.deserialize(src.shot),
            src.practice,
            Difficulty.from(src.difficulty)
        );

        if (src.end_info) {
            ret.#cleared = src.end_info[0];
            ret.#endTime = new Date(src.end_info[1]);
        }

        ret.#misses = src.misses.map((pair) => [new Date(pair[0]), StageLocation.deserialize(pair[1])]);
        ret.#bombs = src.bombs.map((pair) => [new Date(pair[0]), StageLocation.deserialize(pair[1])]);
        ret.#breaks = src.breaks.map((pair) => [new Date(pair[0]), StageLocation.deserialize(pair[1])]);
        ret.#locationsSeen = src.locations_seen.map(StageLocation.deserialize);
        ret.#score = src.score;
        ret.#continues = src.continues;
        ret.#events = src.events.map(GameEvent.deserialize);

        return ret;
    }

    /**
     * 
     * @param {StartGameEvent} startEvent 
     */
    static fromStartEvent(startEvent) {
        Game.#internalOnly = true;
        var ret = new Game(
            startEvent.time,
            startEvent.location,
            startEvent.shot,
            startEvent.practice,
            startEvent.difficulty
        );

        ret.#events.push(startEvent);

        return ret;
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

    /** @returns {[Date, StageLocation][]} */
    get misses() {
        return this.#misses;
    }

    /** @returns {[Date, StageLocation][]} */
    get bombs() {
        return this.#bombs;
    }

    /** @returns {[Date, StageLocation][]} */
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

    /** @returns {number} */
    get score() {
        return this.#score;
    }

    /** @returns {number} */
    get continues() {
        return this.#continues;
    }

    /** @returns {GameEvent[]} */
    get events() {
        return this.#events;
    }

    /** @returns {StartGameEvent} */
    get startEvent() {
        return this.#events.find((ev) => ev instanceof StartGameEvent);
    }

    /** @returns {boolean} */
    get isThprac() {
        return this.#practice && (this.startEvent.location.sectionType !== "start");
    }

    /**
     * @param {StageLocation} location 
     */
    #addSeenLocation(location) {
        if (this.#locationsSeen.findIndex((elem) => elem.equals(location)) >= 0) return;
        this.#locationsSeen.push(location);
        this.#locationsSeen.sort((a, b) => a.compare(b));
    }

    forceEnd() {
        if (!this.ended) {
            this.#endTime = new Date();
            this.#cleared = false;
        }
    }

    /**
     * Update this game's state with a new event.
     * @param {GameEvent} event 
     */
    addEvent(event) {
        this.#events.push(event);
        if (event instanceof EnterSectionEvent) {
            this.#addSeenLocation(event.location);
            this.#currentLocation = event.location;
        } else if (event instanceof EndGameEvent) {
            this.#addSeenLocation(event.location);
            this.#currentLocation = event.location;
            this.#endTime = event.time;
            this.#cleared = event.cleared;
        } else if (event instanceof MissEvent) {
            this.#addSeenLocation(event.location);
            this.#misses.push([event.time, event.location]);
        } else if (event instanceof BombEvent) {
            this.#addSeenLocation(event.location);
            this.#bombs.push([event.time, event.location]);
        } else if (event instanceof BorderEndEvent) {
            this.#addSeenLocation(event.location);
            if (event.broken) {
                this.#breaks.push([event.time, event.location]);
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