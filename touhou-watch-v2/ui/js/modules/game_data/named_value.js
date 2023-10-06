export class NamedValue {
    /** @type {number} */
    #value;

    /** @type {string} */
    #name;

    /**
     * 
     * @param {number} value 
     * @param {string} name 
     */
    constructor (value, name) {
        this.#value = value;
        this.#name = name;
    }

    /**
     * Deserialize a NamedValue directly from Rust data.
     * 
     * @param {Object} src 
     * @returns {NamedValue}
     */
    static deserialize(src) {
        return new NamedValue(src.value, src.name);
    }

    /** @returns {string} */
    get name() {
        return this.#name;
    }

    /** @returns {number} */
    get value() {
        return this.#value;
    }

    /**
     * 
     * @param {NamedValue} other 
     * @returns {boolean}
     */
    compare(other) {
        if (!(other instanceof NamedValue)) {
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

export class GameId extends NamedValue {
    /** @type {string} */
    #abbreviation;

    constructor (value, name, abbreviation) {
        super(value, name);
        this.#abbreviation = abbreviation;
    }

    /**
     * Deserialize a GameId directly from Rust data.
     * 
     * @param {Object} src 
     * @returns {GameId}
     */
    static deserialize(src) {
        return new GameId(src.value, src.name, src.abbreviation);
    }

    /** @returns {string} */
    get abbreviation() {
        return this.#abbreviation;
    }

    /** @returns {string} */
    get cssClass() {
        switch (this.value) {
            case 7: return "pcb";
            case 8: return "in";
            case 10: return "mof";
            default: return "";
        }
    }
}
