import Game from './game.js';

/**
 * @param {string} classList 
 * @returns {HTMLDivElement}
 */
function createDivWithClasses(classList) {
    var ret = document.createElement("div");
    ret.className = classList;
    return ret;
}


class LocationListRow {
    static #ROW_CELLS = 5;

    /** @type {HTMLDivElement} */
    #container;

    /** @type {HTMLDivElement} */
    #listElem;

    /**
     * 
     * @param {string} header 
     */
    constructor (header) {
        this.#container = createDivWithClasses("game-details-row game-location-list-container");
        this.#listElem = createDivWithClasses("game-location-list");

        var headerElem = createDivWithClasses("game-location-list-header game-location-list-entry");
        headerElem.innerText = header;
        
        this.#container.appendChild(headerElem);
        this.#container.appendChild(this.#listElem);
        this.shown = false;
    }

    /** @returns {HTMLDivElement} */
    get rootElement() {
        return this.#container;
    }

    /** @returns {boolean} */
    get shown() {
        return this.#container.style.display !== "none";
    }

    /** @param {boolean} value */
    set shown(value) {
        this.#container.style.display = value ? null : "none";
    }

    /**
     * Update this element to display a new list of locations.
     * @param {string[]} locations 
     */
    update(locations) {
        if (locations.length > 0) {
            let locs = locations.slice();
            while (locs.length % LocationListRow.#ROW_CELLS != 0) {
                locs.push("");
            }

            Element.prototype.replaceChildren.apply(this.#listElem, locs.map((loc) => {
                var elem = createDivWithClasses("game-location-list-entry");
                elem.innerText = loc;
                return elem;
            }));

            this.shown = true;
        } else {
            this.shown = false;
        }
    }
}

export default class GameDisplay {
    /** @type {number} */
    #gameNumber;

    /** @type {Game} */
    #game;

    /** @type {HTMLDivElement} */
    #container;

    /** @type {HTMLDivElement} */
    #indexElem;

    /** @type {HTMLDivElement} */
    #characterElem;

    /** @type {HTMLDivElement} */
    #modeElem;

    /** @type {HTMLDivElement} */
    #locationElem;

    /** @type {HTMLDivElement} */
    #resultElem;

    /** @type {HTMLDivElement} */
    #locationCountElem;

    /** @type {HTMLDivElement} */
    #durationElem;

    /** @type {LocationListRow} */
    #missRow;

    /** @type {LocationListRow} */
    #bombRow;

    /** @type {LocationListRow} */
    #breakRow;

    /**
     * 
     * @param {number} gameNumber 
     * @param {Game} game 
     */
    constructor (gameNumber, game) {
        this.#gameNumber = gameNumber;
        this.#game = game;

        this.#container = createDivWithClasses("game-container");

        this.#indexElem = createDivWithClasses("game-details-entry game-details-index");
        this.#characterElem = createDivWithClasses("game-details-entry game-details-character");
        this.#modeElem = createDivWithClasses("game-details-entry game-details-mode");
        this.#locationElem = createDivWithClasses("game-details-entry game-details-location");
        this.#resultElem = createDivWithClasses("game-details-entry game-details-result");
        this.#locationCountElem = createDivWithClasses("game-details-entry game-details-location-count");
        this.#durationElem = createDivWithClasses("game-details-entry game-details-duration");
        
        var summaryRow = createDivWithClasses("game-details-row game-summary-row");
        summaryRow.replaceChildren(
            this.#indexElem,
            this.#characterElem,
            this.#modeElem,
            this.#locationElem,
            this.#resultElem,
            this.#locationCountElem,
            this.#durationElem
        );

        this.#missRow = new LocationListRow("Misses");
        this.#bombRow = new LocationListRow("Bombs");
        this.#breakRow = new LocationListRow("Border Breaks");
        
        this.#container.replaceChildren(
            summaryRow,
            this.#missRow.rootElement,
            this.#bombRow.rootElement,
            this.#breakRow.rootElement
        );

        this.#indexElem.innerText = gameNumber;
        
        this.update();
    }

    /** @returns {number} */
    get gameNumber() {
        return this.#gameNumber;
    }
    
    /** @param {number} value */
    set gameNumber(value) {
        this.#gameNumber = value;
        this.#indexElem.innerText = value;
    }

    /** @returns {Game} */
    get game() {
        return this.#game;
    }

    /** @param {Game} value */
    set game(value) {
        if (!(value instanceof Game)) throw new TypeError("value must be a Game");
        this.#game = value;
        this.update();
    }

    /** @returns {HTMLDivElement} */
    get rootElement() {
        return this.#container;
    }

    updateTime() {
        var effEndTime = this.#game.ended ? this.#game.endTime.valueOf() : Date.now();
        var duration = effEndTime - this.#game.startTime.valueOf();
        
        var total_sec = Math.floor(duration / 1000);
        var ds = Math.floor((duration % 1000) / 100);
        var m = Math.floor(total_sec / 60);
        var s = Math.floor(total_sec % 60);
        var tm_string = (
            ((m < 10) ? ("0" + m.toFixed(0)) : m.toFixed(0))
            + ":" + ((s < 10) ? ("0" + s.toFixed(0)) : s.toFixed(0))
            + "." + ds
        );
    
        this.#durationElem.innerText = tm_string;
    }

    update() {
        var game = this.#game;

        this.#characterElem.innerText = game.shot;
        this.#modeElem.innerText = game.difficulty + (game.practice ? " Practice" : "");

        if (game.ended) {
            if (game.cleared) {
                if (this.game.practice) {
                    this.#locationElem.innerText = "Cleared " + game.currentStage;
                } else {
                    this.#locationElem.innerText = "All cleared";
                }
            } else {
                this.#locationElem.innerText = "Ended at " + game.currentLocation;
            }
        } else {
            this.#locationElem.innerText = "Currently at " + game.currentLocation;
        }

        var result = game.resultAbbreviation(false);
        var colorClass = "game-state-normal";
        if (game.ended && !game.practice) {
            if (!game.cleared) result = "Failed";
            colorClass = game.cleared ? "game-state-cleared" : "game-state-failed";
        } else if (!game.ended) {
            result = "Running (" + result + ")";
            colorClass = "game-state-running";
        }

        this.#resultElem.innerText = result;
        this.#locationCountElem.innerText = game.locationsSeen.length;

        this.#missRow.update(game.misses.map((pair) => pair[1]));
        this.#bombRow.update(game.bombs.map((pair) => pair[1]));
        this.#breakRow.update(game.breaks.map((pair) => pair[1]));

        this.#container.classList.remove("game-state-normal", "game-state-running", "game-state-cleared", "game-state-failed");
        this.#container.classList.add(colorClass);

        this.#container.scrollIntoView(false);
        this.updateTime();
    }
}
