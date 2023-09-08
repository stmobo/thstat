import Game from '../game.js';
import { formatDuration } from '../utils.js';
import Display from './display.js';

/**
 * @param {string} classList 
 * @returns {HTMLDivElement}
 */
function createDivWithClasses(classList) {
    var ret = document.createElement("div");
    ret.className = classList;
    return ret;
}


class LocationListRow extends Display {
    static #ROW_CELLS = 5;

    /** @type {HTMLDivElement} */
    #listElem;

    /**
     * 
     * @param {string} header 
     */
    constructor (header) {
        super(createDivWithClasses("game-details-row game-location-list-container"));

        this.#listElem = createDivWithClasses("game-location-list");

        var headerElem = createDivWithClasses("game-location-list-header game-location-list-entry");
        headerElem.innerText = header;
        
        this.rootElement.appendChild(headerElem);
        this.rootElement.appendChild(this.#listElem);
        this.shown = false;
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

export default class GameDisplay extends Display {
    /** @type {number} */
    #gameNumber;

    /** @type {Game} */
    #game;

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
        super(createDivWithClasses("game-container"));

        this.#gameNumber = gameNumber;
        this.#game = game;

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
        
        this.rootElement.replaceChildren(
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

    updateTime() {
        var effEndTime = this.#game.ended ? this.#game.endTime.valueOf() : Date.now();
        this.#durationElem.innerText = formatDuration(effEndTime - this.#game.startTime.valueOf());
    }

    update() {
        var game = this.#game;

        this.#characterElem.innerText = game.shot;
        
        if (game.practice) {
            this.#modeElem.innerText = game.difficulty + " Practice" + (game.isThprac ? " (thprac)" : "");
        } else {
            this.#modeElem.innerText = game.difficulty;
        }

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

        var prevMissShown = this.#missRow.shown;
        var prevBombShown = this.#bombRow.shown;
        var prevBreakShown = this.#breakRow.shown;

        this.#missRow.update(game.misses.map((pair) => pair[1]));
        this.#bombRow.update(game.bombs.map((pair) => pair[1]));
        this.#breakRow.update(game.breaks.map((pair) => pair[1]));

        this.rootElement.classList.remove("game-state-normal", "game-state-running", "game-state-cleared", "game-state-failed");
        this.rootElement.classList.add(colorClass);

        this.updateTime();

        if (
            (this.#missRow.shown !== prevMissShown)
            || (this.#bombRow.shown !== prevBombShown)
            || (this.#breakRow.shown !== prevBreakShown)
        ) {
            this.scrollIntoView();
        }
    }
}
