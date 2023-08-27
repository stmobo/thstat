export default class Difficulty {
    /** @type {number} */
    #id;

    static #initializing = false;

    static #easy;
    static #normal;
    static #hard;
    static #lunatic;
    static #extra;
    static #phantasm;

    /** @returns {Difficulty} */
    static get EASY() {
        return Difficulty.#easy;
    }

    /** @returns {Difficulty} */
    static get NORMAL() {
        return Difficulty.#normal;
    }

    /** @returns {Difficulty} */
    static get HARD() {
        return Difficulty.#hard;
    }

    /** @returns {Difficulty} */
    static get LUNATIC() {
        return Difficulty.#lunatic;
    }

    /** @returns {Difficulty} */
    static get EXTRA() {
        return Difficulty.#extra;
    }

    /** @returns {Difficulty} */
    static get PHANTASM() {
        return Difficulty.#phantasm;
    }

    constructor (id) {
        if (!Difficulty.#initializing) {
            throw new TypeError("Difficulty instances cannot be created directly, use Difficulty.from instead");
        }

        this.#id = id;
    }

    /**
     * @param {number} id
     * @returns {Difficulty}
     */
    static from(id) {
        switch (id) {
        case 0: return Difficulty.#easy;
        case 1: return Difficulty.#normal;
        case 2: return Difficulty.#hard;
        case 3: return Difficulty.#lunatic;
        case 4: return Difficulty.#extra;
        case 5: return Difficulty.#phantasm;
        default: throw new RangeError("invalid difficulty ID " + id);
        }
    }

    /** @returns {number} */
    get id() {
        return this.#id;
    }

    /** @returns {string} */
    get name() {
        switch (this.#id) {
        case 0: return "Easy";
        case 1: return "Normal";
        case 2: return "Hard";
        case 3: return "Lunatic";
        case 4: return "Extra";
        case 5: return "Phantasm";
        default: return null; // shouldn't happen
        }
    }

    /**
     * 
     * @param {Difficulty} other 
     * @returns {boolean}
     */
    compare(other) {
        if (!(other instanceof Difficulty)) {
            return 1;
        }

        return this.id - other.id;
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

    static {
        Difficulty.#initializing = true;
        Difficulty.#easy = new Difficulty(0);
        Difficulty.#normal = new Difficulty(1);
        Difficulty.#hard = new Difficulty(2);
        Difficulty.#lunatic = new Difficulty(3);
        Difficulty.#extra = new Difficulty(4);
        Difficulty.#phantasm = new Difficulty(5);
        Difficulty.#initializing = false;
    }
}
