export default class Stage {
    /** @type {number} */
    #id;

    static #initializing = false;

    static #one;
    static #two;
    static #three;
    static #four;
    static #five;
    static #six;
    static #extra;
    static #phantasm;

    /** @returns {Stage} */
    static get ONE() {
        return Stage.#one;
    }

    /** @returns {Stage} */
    static get TWO() {
        return Stage.#two;
    }

    /** @returns {Stage} */
    static get THREE() {
        return Stage.#three;
    }

    /** @returns {Stage} */
    static get FOUR() {
        return Stage.#four;
    }

    /** @returns {Stage} */
    static get FIVE() {
        return Stage.#five;
    }

    /** @returns {Stage} */
    static get SIX() {
        return Stage.#six;
    }

    /** @returns {Stage} */
    static get EXTRA() {
        return Stage.#extra;
    }

    /** @returns {Stage} */
    static get PHANTASM() {
        return Stage.#phantasm;
    }

    constructor (id) {
        if (!Stage.#initializing) {
            throw new TypeError("Stage instances cannot be created directly, use Stage.from instead");
        }

        this.#id = id;
    }

    /**
     * @param {number} id
     * @returns {Stage}
     */
    static from(id) {
        switch (id) {
        case 0: return Stage.#one;
        case 1: return Stage.#two;
        case 2: return Stage.#three;
        case 3: return Stage.#four;
        case 4: return Stage.#five;
        case 5: return Stage.#six;
        case 6: return Stage.#extra;
        case 7: return Stage.#phantasm;
        default: throw new RangeError("invalid stage ID " + id);
        }
    }

    /** @returns {number} */
    get id() {
        return this.#id;
    }

    /** @returns {string} */
    get name() {
        switch (this.#id) {
        case 0: return "Stage 1";
        case 1: return "Stage 2";
        case 2: return "Stage 3";
        case 3: return "Stage 4";
        case 4: return "Stage 5";
        case 5: return "Stage 6";
        case 6: return "Extra Stage";
        case 7: return "Phantasm Stage";
        default: return null; // shouldn't happen
        }
    }

    /**
     * 
     * @param {Stage} other 
     * @returns {boolean}
     */
    compare(other) {
        if (!(other instanceof Stage)) {
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
        Stage.#initializing = true;
        Stage.#one = new Stage(0);
        Stage.#two = new Stage(1);
        Stage.#three = new Stage(2);
        Stage.#four = new Stage(3);
        Stage.#five = new Stage(4);
        Stage.#six = new Stage(5);
        Stage.#extra = new Stage(6);
        Stage.#phantasm = new Stage(7);
        Stage.#initializing = false;
    }
}
