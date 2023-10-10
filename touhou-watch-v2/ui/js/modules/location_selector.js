import Display from "./display.js";
import { newElement } from "./utils.js";

const { invoke } = window.__TAURI__;

class LocationInfo {
    /** @type {number} */
    #game;

    /** @type {number} */
    #index;
    
    /** @type {string} */
    #name;
    
    /** @type {number} */
    #stageId;
    
    /** @type {string} */
    #stageName;

    constructor (index, src) {
        this.#index = index;
        this.#game = src.game;
        this.#name = src.name;
        this.#stageId = src.stage[0];
        this.#stageName = src.stage[1];
    }

    /** @returns {number} */
    get game() {
        return this.#game;
    }

    /** @returns {number} */
    get index() {
        return this.#index;
    }

    /** @returns {string} */
    get name() {
        return this.#name;
    }

    /** @returns {number} */
    get stageId() {
        return this.#stageId;
    }

    /** @returns {string} */
    get stageName() {
        return this.#stageName;
    }
}

/**
 * 
 * @param {number | string} value 
 * @param {string} name 
 * @returns {HTMLOptionElement}
 */
function newOption(value, name) {
    var ret = newElement("option", { text: name });
    ret.setAttribute("value", value);
    return ret;
}

class LocationSelector extends Display {
    /** @type {number?} */
    #game = null;

    /** @type {LocationInfo[]} */
    #opts = [];

    /** @type {number?} */
    #selectedStage = null;

    /** @type {number?} */
    #selectedSection = null;

    /** @type {HTMLSelectElement} */
    #stageElem;

    /** @type {HTMLSelectElement} */
    #sectionElem;

    /** @type {(index: number?) => void} */
    #valueCallback;

    /**
     * 
     * @param {string} label 
     * @param {(index: number?) => void} valueCallback 
     */
    constructor (label, valueCallback) {
        super(newElement("div", { className: "location-select-container" }));

        let labelElem = newElement("label", { text: label, className: "location-select-label" });
        this.#stageElem = newElement("select", { className: "location-select-stage" })
        this.#sectionElem = newElement("select", { className: "location-select-section" })

        this.rootElement.replaceChildren(labelElem, this.#stageElem, this.#sectionElem);

        this.#valueCallback = valueCallback;

        this.#stageElem.addEventListener("change", (ev) => {
            this.#selectStage(this.#stageElem.value);
        });

        this.#sectionElem.addEventListener("change", (ev) => {
            this.#selectSection(this.#sectionElem.value);
        });
        
        this.#stageElem.replaceChildren(newOption("", ""));
        this.#stageElem.value = "";
        this.#stageElem.disabled = true;
        
        this.#sectionElem.replaceChildren(newOption("", ""));
        this.#sectionElem.value = "";
        this.#sectionElem.disabled = true;
        this.#sectionElem.style.display = "none";
    }

    /** @returns {number?} */
    get value() {
        return this.#selectedSection;
    }

    /**
     * 
     * @param {LocationInfo[]} options 
     */
    #updateOptions(options) {
        if (options.length > 0) {
            /** @type {{[id: number]: string}} */
            let stages = {};
    
            for (let loc of options) {
                stages[loc.stageId] = loc.stageName;
            }
    
            let stageElems = Object.entries(stages).map((kv) => newOption(kv[0], kv[1]));
            stageElems.unshift(newOption("", "Select stage..."));
            Element.prototype.replaceChildren.apply(this.#stageElem, stageElems);
    
            this.#stageElem.value = "";
            this.#stageElem.disabled = false;
        } else {
            this.#stageElem.replaceChildren(newOption("", ""));
            this.#stageElem.value = "";
            this.#stageElem.disabled = true;
        }

        this.#selectStage(null);
        this.#opts = options;
    }

    #selectStage(stageId) {
        if (stageId === "" || stageId === null || stageId === undefined) {
            stageId = null;
        } else {
            stageId = parseInt(stageId, 10);
        }

        if (this.#selectedStage !== stageId) {
            if (stageId === null) {
                this.#sectionElem.replaceChildren(newOption("", ""));
                this.#sectionElem.value = "";
                this.#sectionElem.disabled = true;
                this.#sectionElem.style.display = "none";
            } else {
                let elems = this.#opts.filter((loc) => loc.stageId === stageId).map((loc) => newOption(loc.index, loc.name));
                elems.unshift(newOption("", "Select section..."));
                Element.prototype.replaceChildren.apply(this.#sectionElem, elems);

                this.#sectionElem.value = "";
                this.#sectionElem.disabled = false;
                this.#sectionElem.style.display = null;
            }

            this.#selectedStage = stageId;
            this.#selectSection(null);
        }
    }

    #selectSection(sectionId) {
        if (sectionId === "" || sectionId === null || sectionId === undefined) {
            sectionId = null;
        } else {
            sectionId = parseInt(sectionId, 10);
        }

        if (this.#selectedSection !== sectionId) {
            this.#selectedSection = sectionId;
            this.#valueCallback(sectionId);
        }
    }

    /**
     * 
     * @param {number | string | null} gameId 
     */
    update (gameId) {
        if (gameId === null || gameId === undefined || gameId === "") {
            gameId = null;
        } else {
            gameId = parseInt(gameId, 10);
        }

        if (this.#game !== gameId) {
            if (gameId === null) {
                this.#updateOptions([]);
                this.#game = null;
            } else {
                invoke('get_locations', { gameId: gameId }).then((locations) => {
                    this.#updateOptions(locations.map((obj, idx) => new LocationInfo(idx, obj)));
                    this.#game = gameId;
                });
            }
        }
    }
}

export class TrackingRangeSelector extends Display {
    /** @type {number?} */
    #game;

    /** @type {LocationSelector} */
    #startElem;

    /** @type {LocationSelector} */
    #endElem;

    constructor () {
        super(newElement("div", { className: "track-range-container" }));

        this.#startElem = new LocationSelector("Start:", (id) => this.#rangeUpdated(id, this.#endElem.value));
        this.#endElem = new LocationSelector("End:", (id) => this.#rangeUpdated(this.#startElem.value, id));
        this.rootElement.replaceChildren(this.#startElem.rootElement, this.#endElem.rootElement);
    }

    /**
     * 
     * @param {number?} start 
     * @param {number?} end 
     */
    #rangeUpdated(start, end) {
        if (start !== null && end !== null && this.#game !== null) {
            console.log(start, end, this.#game);
            invoke('start_tracking', { gameId: this.#game, startIndex: start, endIndex: end });
        } else {
            invoke('end_tracking');
        }
    }

    /**
     * 
     * @param {number | string | null} gameId 
     */
    update (gameId) {
        if (gameId === null || gameId === undefined || gameId === "") {
            gameId = null;
        } else {
            gameId = parseInt(gameId, 10);
        }

        if (this.#game !== gameId) {
            this.#game = gameId;
            this.#startElem.update(this.#game);
            this.#endElem.update(this.#game);
        }
    }
}
