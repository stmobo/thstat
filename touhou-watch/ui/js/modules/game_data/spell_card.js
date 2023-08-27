const { invoke } = window.__TAURI__;

import Stage from "./stage.js";
import Difficulty from "./difficulty.js";

export default class SpellCard {
    /** @type {number} */
    #id;

    static #dataInitialized = false;

    /** @type {{ name: string, difficulty: Difficulty, stage: Stage, is_midboss: boolean }[]} */
    static #cardData;

    /** @param {number} id */
    constructor (id) {
        if (!SpellCard.#dataInitialized) {
            throw new TypeError("Spell card data not initialized yet");
        }

        if (!Number.isInteger(id) || id <= 0 || id > SpellCard.#cardData.length) {
            throw new RangeError("Invalid card ID " + id);
        }

        this.#id = id;
    }

    /**
     * 
     * @returns {Promise<void>}
     */
    static initData() {
        /* TODO: make this static data */
        return invoke('load_spellcard_data', {}).then((data) => {
            data.forEach((elem) => {
                elem.difficulty = Difficulty.from(elem.difficulty);
                elem.stage = Stage.from(elem.stage);
            });

            SpellCard.#cardData = data;
            SpellCard.#dataInitialized = true;
        });
    }

    /**
     * Deserialize a SpellCard from raw event data.
     * 
     * @param {[number, number]} src 
     * @returns {SpellCard}
     */
    static deserialize(src) {
        if (src[0] !== 7) throw new TypeError("Game ID incorrect (expected 7, got " + src[0] + ")");
        return new SpellCard(src[1]);
    }

    /** @returns {string} */
    get key() {
        if (this.id < 10) {
            return "00" + this.id;
        } else if (this.id < 100) {
            return "0" + this.id;
        } else {
            return this.id.toString();
        }
    }

    /** @returns {number} */
    get id() {
        return this.#id;
    }

    /** @returns {string} */
    get name() {
        return SpellCard.#cardData[this.id - 1].name;
    }

    /** @returns {Difficulty} */
    get difficulty() {
        return SpellCard.#cardData[this.id - 1].difficulty;
    }

    /** @returns {Stage} */
    get stage() {
        return SpellCard.#cardData[this.id - 1].stage;
    }

    /** @returns {boolean} */
    get is_midboss() {
        return SpellCard.#cardData[this.id - 1].is_midboss;
    }

    /** 
     * @param {SpellCard | number} other
     * @returns {boolean}
     */
    equals(other) {
        if (other instanceof SpellCard) {
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

