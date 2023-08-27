export default class ShotType {
    /** @type {number} */
    #id;

    constructor (id) {
        if (!Number.isInteger(id) || id < 0 || id > 5) {
            throw new RangeError("invalid shot type ID " + id);
        }

        this.#id = id;
    }

    /**
     * Deserialize a ShotType from raw event data.
     * 
     * @param {[number, number]} src 
     * @returns {ShotType}
     */
    static deserialize(src) {
        if (src[0] != 7) throw new TypeError("Game ID incorrect (expected 7, got " + src[0] + ")");
        return new ShotType(src[1]);
    }

    /** @returns {number} */
    get id() {
        return this.#id;
    }

    /** @returns {string} */
    get name() {
        switch (this.#id) {
        case 0: return "Reimu A";
        case 1: return "Reimu B";
        case 2: return "Marisa A";
        case 3: return "Marisa B";
        case 4: return "Sakuya A";
        case 5: return "Sakuya B";
        default: return null; // shouldn't happen
        }
    }

    /**
     * 
     * @param {ShotType} other 
     * @returns {boolean}
     */
    compare(other) {
        if (!(other instanceof ShotType)) {
            return 1;
        }

        return this.id - other.id;
    }

    /** 
     * @param {ShotType | number} other
     * @returns {boolean}
     */
    equals(other) {
        if (other instanceof ShotType) {
            return this.id === other.id;
        } else if (typeof other === "number") {
            return this.id === other;
        } else {
            return false;
        }
    }

    valueOf() {
        return this.#id;
    }

    toString() {
        return this.name;
    }

    [Symbol.toPrimitive](hint) {
        if (hint === 'number') {
            return this.#id;
        } else {
            return this.name;
        }
    }
}
