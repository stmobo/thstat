import { StageLocation } from './locations.js';
import SpellCard from './spell_card.js';
import ShotType from './shot_type.js';
import Difficulty from './difficulty.js';
import Stage from './stage.js';


var DESERIALIZE_INTERNAL_FLAG = false;

export class GameEvent {
    /** @type {Date} */
    #time;

    /**
     * @param {number} type_id
     * @param {number} time 
     */
    constructor (time) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("GameEvent instances cannot be constructed directly");
        }

        this.#time = new Date(time);
    }

    /**
     * Deserialize a game event from raw event data.
     * Returns a subclass of GameEvent.
     */
    static deserialize(src) {
        try {
            DESERIALIZE_INTERNAL_FLAG = true;
            switch (src.event) {
            case "start_game": return new StartGameEvent(src);
            case "end_game": return new EndGameEvent(src);
            case "stage_cleared": return new StageClearedEvent(src);
            case "enter_section": return new EnterSectionEvent(src);
            case "miss": return new MissEvent(src);
            case "bomb": return new BombEvent(src);
            case "finish_spell": return new FinishSpellEvent(src);
            case "border_start": return new BorderStartEvent(src);
            case "border_end": return new BorderEndEvent(src);
            case "pause": return new PauseEvent(src);
            case "unpause": return new UnpauseEvent(src);
            default: throw new TypeError("Invalid event type " + src.event);
            }
        } finally {
            DESERIALIZE_INTERNAL_FLAG = false;
        }
    }

    /** @returns {Date} */
    get time() {
        return this.#time;
    }

    /** @returns {StageLocation?} */
    get location() {
        return null;
    }
}

export class StartGameEvent extends GameEvent {
    /** @type {ShotType} */
    #shot;
    
    /** @type {Difficulty} */
    #difficulty;

    /** @type {StageLocation} */
    #location;

    /** @type {boolean} */
    #practice;

    /** @type {number} */
    #lives;

    /** @type {number} */
    #bombs;

    /** @type {number} */
    #power;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("StartGameEvent instances cannot be constructed directly");
        }
        
        super(src.time);

        this.#shot = ShotType.deserialize(src.character);
        this.#difficulty = Difficulty.from(src.difficulty);
        this.#location = StageLocation.deserialize(src.location);

        if (typeof src.practice !== "boolean") throw new TypeError("practice is not a boolean");
        this.#practice = src.practice;

        if ((typeof src.lives !== "number") || !Number.isInteger(src.lives)) throw new TypeError("lives is not an integer (got " + src.lives + ")");
        if (src.lives < 0 || src.lives > 8) throw new TypeError("invalid number of lives (got " + src.lives + ", expected 0-8)");
        this.#lives = src.lives;

        if ((typeof src.bombs !== "number") || !Number.isInteger(src.bombs)) throw new TypeError("bombs is not an integer (got " + src.bombs + ")");
        if (src.bombs < 0 || src.bombs > 8) throw new TypeError("invalid number of bombs (got " + src.bombs + ", expected 0-8)");
        this.#bombs = src.bombs;

        if ((typeof src.power !== "number") || !Number.isInteger(src.power)) throw new TypeError("power is not an integer (got " + src.power + ")");
        if (src.power < 0 || src.power > 128) throw new TypeError("invalid power (got " + src.power + ", expected 0-128)");
        this.#power = src.power;
    }

    /** @returns {ShotType} */
    get shot() {
        return this.#shot;
    }

    /** @returns {Difficulty} */
    get difficulty() {
        return this.#difficulty;
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {boolean} */
    get practice() {
        return this.#practice;
    }

    /** @returns {number} */
    get lives() {
        return this.#lives;
    }

    /** @returns {number} */
    get bombs() {
        return this.#bombs;
    }

    /** @returns {number} */
    get power() {
        return this.#power;
    }

    /** @returns {string} */
    toString() {
        if (this.#practice) {
            return "Started " + this.#difficulty + " practice as " + this.#shot + " at " + this.#location;
        } else {
            return "Started " + this.#difficulty + " run as " + this.#shot;
        }
    }
}

export class EndGameEvent extends GameEvent {
    /** @type {StageLocation} */
    #location;

    /** @type {number} */
    #misses;

    /** @type {number} */
    #bombs;

    /** @type {number} */
    #continues;

    /** @type {boolean} */
    #cleared;

    /** @type {boolean} */
    #retrying;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("EndGameEvent instances cannot be constructed directly");
        }
        
        super(src.time);

        this.#location = StageLocation.deserialize(src.location);

        if ((typeof src.misses !== "number") || !Number.isInteger(src.misses)) throw new TypeError("misses is not an integer (got " + src.misses + ")");
        if (src.misses < 0) throw new TypeError("invalid miss count (got " + src.misses + ", expected non-negative integer)");
        this.#misses = src.misses;

        if ((typeof src.bombs !== "number") || !Number.isInteger(src.bombs)) throw new TypeError("bombs is not an integer (got " + src.bombs + ")");
        if (src.bombs < 0) throw new TypeError("invalid bomb count (got " + src.bombs + ", expected non-negative integer)");
        this.#bombs = src.bombs;

        if ((typeof src.continues !== "number") || !Number.isInteger(src.continues)) throw new TypeError("continues is not an integer (got " + src.continues + ")");
        if (src.continues < 0) throw new TypeError("invalid continue count (got " + src.continues + ", expected non-negative integer)");
        this.#continues = src.continues;

        if (typeof src.cleared !== "boolean") throw new TypeError("cleared is not a boolean");
        this.#cleared = src.cleared;

        if (typeof src.retrying !== "boolean") throw new TypeError("retrying is not a boolean");
        this.#retrying = src.retrying;
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {boolean} */
    get cleared() {
        return this.#cleared;
    }

    /** @returns {boolean} */
    get retrying() {
        return this.#retrying;
    }

    /** @returns {number} */
    get misses() {
        return this.#misses;
    }

    /** @returns {number} */
    get bombs() {
        return this.#bombs;
    }

    /** @returns {number} */
    get continues() {
        return this.#continues;
    }

    /** @returns {string} */
    toString() {
        var ret = "";
        if (this.#cleared) {
            ret = "Cleared game at ";
        } else if (this.#retrying) {
            ret = "Retried game at ";
        } else {
            ret = "Ended game at ";
        }

        ret += this.#location;

        if (this.#misses == 0 && this.#bombs == 0) {
            ret += " with no misses or bombs used";
        } else {
            if (this.#misses > 0) {
                ret += " with " + (this.#misses == 1 ? "1 miss" : this.#misses + " misses");
            }

            if (this.#bombs > 0) {
                if (this.#misses > 0 && this.#continues > 0) {
                    ret += ", ";
                } else if (this.#misses > 0) {
                    ret += " and ";
                } else {
                    ret += " with ";
                }

                ret += this.#bombs == 1 ? "1 bomb" : this.#bombs + " bombs";
            }

            if (this.#continues > 0) {
                if (this.#misses > 0 && this.#bombs > 0) {
                    ret += ", and ";
                } else if (this.#misses > 0) {
                    ret += " and ";
                } else {
                    ret += " with ";
                }

                ret += this.#continues == 1 ? "1 continue used" : this.#continues + " continues used";
            }
        }

        return ret;
    }
}

export class StageClearedEvent extends GameEvent {
    /** @type {Stage} */
    #stage;
    
    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("StageClearEvent instances cannot be constructed directly");
        }
        
        super(src.time);
        this.#stage = Stage.from(src.stage);
    }

    /** @returns {Stage} */
    get stage() {
        return this.#stage;
    }

    /** @returns {string} */
    toString() {
        return "Cleared " + this.#stage;
    }
}

export class EnterSectionEvent extends GameEvent {
    /** @type {StageLocation} */
    #location;

    /** @type {number} */
    #lives;

    /** @type {number} */
    #bombs;

    /** @type {number} */
    #power;

    /** @type {number} */
    #continues;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("EnterSectionEvent instances cannot be constructed directly");
        }
        
        super(src.time);

        this.#location = StageLocation.deserialize(src.location);

        if ((typeof src.lives !== "number") || !Number.isInteger(src.lives)) throw new TypeError("lives is not an integer (got " + src.lives + ")");
        if (src.lives < 0 || src.lives > 8) throw new TypeError("invalid number of lives (got " + src.lives + ", expected 0-8)");
        this.#lives = src.lives;

        if ((typeof src.bombs !== "number") || !Number.isInteger(src.bombs)) throw new TypeError("bombs is not an integer (got " + src.bombs + ")");
        if (src.bombs < 0 || src.bombs > 8) throw new TypeError("invalid number of bombs (got " + src.bombs + ", expected 0-8)");
        this.#bombs = src.bombs;

        if ((typeof src.power !== "number") || !Number.isInteger(src.power)) throw new TypeError("power is not an integer (got " + src.power + ")");
        if (src.power < 0 || src.power > 128) throw new TypeError("invalid power (got " + src.power + ", expected 0-128)");
        this.#power = src.power;

        if ((typeof src.continues !== "number") || !Number.isInteger(src.continues)) throw new TypeError("continues is not an integer (got " + src.continues + ")");
        if (src.continues < 0) throw new TypeError("invalid continue count (got " + src.continues + ", expected non-negative integer)");
        this.#continues = src.continues;
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {number} */
    get lives() {
        return this.#lives;
    }

    /** @returns {number} */
    get bombs() {
        return this.#bombs;
    }

    /** @returns {number} */
    get power() {
        return this.#power;
    }

    /** @returns {number} */
    get continues() {
        return this.#continues;
    }

    /** @returns {string} */
    toString() {
        var ret = "Entering " + this.#location + " with ";
        ret += (this.#lives == 1) ? "1 life, " : this.#lives + " lives, ";
        ret += (this.#bombs == 1) ? "1 bomb, and " : this.#bombs + " bombs, ";
        ret += this.#power + " power, and ";
        ret += (this.#continues == 1) ? "1 continue used" : this.#continues + " continues used";
        return ret;
    }
}

export class MissEvent extends GameEvent {
    /** @type {StageLocation} */
    #location;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("MissEvent instances cannot be constructed directly");
        }
        
        super(src.time);
        this.#location = StageLocation.deserialize(src.location);
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {string} */
    toString() {
        return "Missed at " + this.#location;
    }
}

export class BombEvent extends GameEvent {
    /** @type {StageLocation} */
    #location;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("BombEvent instances cannot be constructed directly");
        }
        
        super(src.time);
        this.#location = StageLocation.deserialize(src.location);
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {string} */
    toString() {
        return "Bombed at " + this.#location;
    }
}

export class FinishSpellEvent extends GameEvent {
    /** @type {SpellCard} */
    #spell;

    /** @type {boolean} */
    #captured;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("FinishSpellEvent instances cannot be constructed directly");
        }
        
        super(src.time);
        this.#spell = SpellCard.deserialize(src.spell);

        if (typeof src.captured !== "boolean") throw new TypeError("captured is not a boolean");
        this.#captured = src.captured;
    }

    /** @returns {SpellCard} */
    get spell() {
        return this.#spell;
    }

    /** @returns {boolean} */
    get captured() {
        return this.#captured;
    }

    /** @returns {string} */
    toString() {
        return (this.#captured ? "Captured" : "Failed") + " spell " + this.#spell;
    }
}

export class BorderStartEvent extends GameEvent {
    /** @type {StageLocation} */
    #location;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("BorderStartEvent instances cannot be constructed directly");
        }
        
        super(src.time);
        this.#location = StageLocation.deserialize(src.location);
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {string} */
    toString() {
        return "Border started at " + this.#location;
    }
}

export class BorderEndEvent extends GameEvent {
    /** @type {StageLocation} */
    #location;

    /** @type {boolean} */
    #broken;

    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("BorderEndEvent instances cannot be constructed directly");
        }
        
        super(src.time);

        this.#location = StageLocation.deserialize(src.location);

        if (typeof src.broken !== "boolean") throw new TypeError("broken is not a boolean");
        this.#broken = src.broken;
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#location;
    }

    /** @returns {boolean} */
    get broken() {
        return this.#broken;
    }

    /** @returns {string} */
    toString() {
        return (this.#broken ? "Broke border" : "Border ended") + " at " + this.#location;
    }
}

export class PauseEvent extends GameEvent {
    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("PauseEvent instances cannot be constructed directly");
        }
        
        super(src.time);
    }

    /** @returns {string} */
    toString() {
        return "Game paused";
    }
}

export class UnpauseEvent extends GameEvent {
    constructor (src) {
        if (!DESERIALIZE_INTERNAL_FLAG) {
            throw new TypeError("UnpauseEvent instances cannot be constructed directly");
        }
        
        super(src.time);
    }

    /** @returns {string} */
    toString() {
        return "Game unpaused";
    }
}