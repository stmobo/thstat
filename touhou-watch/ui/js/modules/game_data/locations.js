import Stage from "./stage.js";
import SpellCard from "./spell_card.js";

/**
 * @typedef {{ type: string, seq?: number, spell?: [number, number] }} SerializedSection
 * @typedef {{ stage: number, section: SerializedSection }} SerializedLocation
 */

export class Section {
    /** @type {number} */
    #section_type;

    /** @type {number?} */
    #seq;

    /** @type {SpellCard?} */
    #spell;

    static #internalOnly = false;
    
    /**
     * @param {number} sec_type 
     * @param {number?} seq 
     * @param {SpellCard?} spell 
     */
    constructor (sec_type, seq, spell) {
        if (!Section.#internalOnly) {
            throw new TypeError("Section instances cannot be created directly");
        }

        if (sec_type < 0 || sec_type > 8) {
            throw new TypeError("Invalid section type " + sec_type);
        }

        this.#section_type = sec_type;
        this.#seq = seq;
        this.#spell = spell;
    }

    /**
     * Deserialize a Section from raw event data.
     * 
     * @param {SerializedSection} src
     * @returns {Section}
     */
    static deserialize(src) {
        try {
            Section.#internalOnly = true;
            switch (src.type) {
            case "start": return new Section(0, null, null);
            case "first_half": return new Section(1, src.seq, null);
            case "midboss_nonspell": return new Section(2, src.seq, null);
            case "midboss_spell": return new Section(3, src.seq, SpellCard.deserialize(src.spell));
            case "second_half": return new Section(4, src.seq, null);
            case "pre_boss": return new Section(5, null, null);
            case "boss_nonspell": return new Section(6, src.seq, null);
            case "boss_spell": return new Section(7, src.seq, SpellCard.deserialize(src.spell));
            case "unknown": return new Section(8, null, null);
            default: throw new TypeError("Invalid section type " + src.type);
            }
        } finally {
            Section.#internalOnly = false;
        }
    }

    /** @returns {string} */
    get key() {
        var ret = this.#section_type;

        if (this.#seq !== null) {
            ret += ":" + this.#seq;

            if (this.#spell !== null) {
                ret += ":" + this.#spell.key;
            }
        }

        return ret;
    }

    /** @returns {string} */
    get type() {
        switch (this.#section_type) {
        case 0: return "start";
        case 1: return "first_half";
        case 2: return "midboss_nonspell";
        case 3: return "midboss_spell";
        case 4: return "second_half";
        case 5: return "pre_boss";
        case 6: return "boss_nonspell";
        case 7: return "boss_spell";
        case 8: return "unknown";
        default: throw new TypeError("Invalid section type " + src.#section_type);
        }
    }

    /** @returns {number?} */
    get section_number() {
        return this.#seq;
    }

    /** @returns {SpellCard?} */
    get spell() {
        return this.#spell;
    }

    /** @returns {boolean} */
    get is_unknown() {
        return this.#section_type === 8;
    }

    /**
     * 
     * @param {Section} other 
     * @returns {number}
     */
    compare(other) {
        if (!(other instanceof Section)) {
            return 1;
        }

        if (this.#section_type != other.#section_type) {
            return this.#section_type - other.#section_type;
        } else if (
            (this.#section_type !== 0)
            && (this.#section_type !== 5)
            && (this.#section_type !== 8)
            && (this.#seq !== other.#seq)
        ) {
            return this.#seq - other.#seq;
        } else if (this.#section_type === 3 || this.#section_type === 4) {
            return this.#spell.id - other.#spell.id;
        } else {
            return 0;
        }
    }

    /**
     * 
     * @param {Section} other 
     * @returns {boolean}
     */
    equals(other) {
        if (!(other instanceof Section)) {
            return false;
        }

        if (!(this.#section_type === other.#section_type) || !(this.#seq === other.#seq)) {
            return false;
        }

        if (this.#spell && other.#spell) {
            return this.#spell.equals(other.#spell);
        } else {
            return (!!this.#spell) === (!!other.#spell);
        }
    }

    /** @returns {string} */
    toString() {
        switch (this.#section_type) {
        case 0: return "Start";
        case 1: return "First Half " + this.#seq;
        case 2: return "Midboss Nonspell " + this.#seq;
        case 3: return this.#spell.toString();
        case 4: return "Second Half " + this.#seq;
        case 5: return "Pre-Boss";
        case 6: return "Boss Nonspell " + this.#seq;
        case 7: return this.#spell.toString();
        default: return "Unknown";
        }
    }
}

export class StageLocation {
    /** @type {Stage} */
    #stage;

    /** @type {Section} */
    #section;

    static #internalOnly = false;
    
    constructor (stage, section) {
        if (!StageLocation.#internalOnly) {
            throw new TypeError("StageLocation instances cannot be created directly");
        }

        this.#stage = stage;
        this.#section = section;
    }

    /**
     * Deserialize a StageLocation from raw event data.
     * 
     * @param {SerializedLocation} src 
     * @returns {StageLocation}
     */
    static deserialize(src) {
        try {
            StageLocation.#internalOnly = true;
            return new StageLocation(
                Stage.from(src.stage),
                Section.deserialize(src.section)
            );
        } finally {
            StageLocation.#internalOnly = false;
        }
    }

    /** @returns {Stage} */
    get stage() {
        return this.#stage;
    }

    /** @returns {Section} */
    get section () {
        return this.#section;
    }

    /** @returns {string} */
    get key() {
        return this.#stage.id + ":" + this.#section.key;
    }

    /** @returns {boolean} */
    get is_unknown() {
        return this.#section.is_unknown;
    }

    /**
     * 
     * @param {StageLocation} other
     * @returns {boolean} 
     */
    compare(other) {
        if (!(other instanceof StageLocation)) {
            return 1;
        }

        if (this.#stage !== other.#stage) {
            return this.#stage.id - other.#stage.id;
        } else {
            return this.#section.compare(other.#section);
        }
    }

    /**
     * 
     * @param {StageLocation} other 
     * @returns {boolean}
     */
    equals(other) {
        if (!(other instanceof StageLocation)) {
            return false;
        }

        return (this.#stage === other.#stage) && this.#section.equals(other.#section);
    }

    /**
     * @returns {string}
     */
    toString() {
        var ret = this.#section.toString();
        if (!this.section.spell) {
            ret = this.#stage.name + " " + ret;
        }
        return ret;
    }
}
