import Game from "./game.js";
import { StageLocation } from "./game_data/locations.js";

/** @typedef {{ location: StageLocation, misses: number, bombs: number, breaks: number, captures: number, attempts: number }} MetricsEntry */

/**
 * @param {number} a
 * @param {number} b
 * @returns {string}
 */
function formatCapRate(a, b) {
    if (b === 0) {
        if (a === 0) {
            return "N/A";
        } else {
            return a + " / 0 (0.0%)";
        }
    }

    var pct = (a / b) * 100.0;
    return a + " / " + b + " (" + pct.toFixed(1) + "%)";
}

/**
 * 
 * @param {string} elemTag 
 * @param {string} classList 
 * @param {string?} text 
 * @returns {HTMLElement}
 */
function createElementWithClasses(elemTag, classList, text = "") {
    var ret = document.createElement(elemTag);
    ret.className = classList;
    if (text) ret.innerText = text;
    return ret;

}

export class Metrics {
    /** @type {{ [key: string]: MetricsEntry }} */
    #metrics;

    /**
     * @param {Game[]} games 
     */
    constructor(games) {
        this.#metrics = {};

        for (let game of games) {
            let captured = {};

            for (let loc of game.locationsSeen) {
                captured[loc.key] = true;
                this.#getEntry(loc).attempts += 1;
            }

            for (let loc of game.misses) {
                captured[loc.key] = false;
                this.#getEntry(loc).misses += 1;
            }

            for (let loc of game.bombs) {
                captured[loc.key] = false;
                this.#getEntry(loc).bombs += 1;
            }

            for (let loc of game.breaks) {
                captured[loc.key] = false;
                this.#getEntry(loc).breaks += 1;
            }

            let currentLocInProgress = !game.ended && (captured[game.currentLocation.key] !== false);
            if (currentLocInProgress) {
                this.#getEntry(game.currentLocation).attempts -= 1;
            }

            Object.entries(captured).filter(
                (p) => p[1] && this.#metrics[p[0]] && !(currentLocInProgress && p[0] === game.currentLocation.key)
            ).forEach(
                (p) => {
                    this.#metrics[p[0]].captures += 1;
                }
            );
        }
    }

    /**
     * @param {StageLocation} location
     * @returns {MetricsEntry}
     */
    #getEntry(location) {
        var key = location.key;
        if (!this.#metrics[key]) {
            this.#metrics[key] = {
                location: location,
                misses: 0,
                bombs: 0,
                breaks: 0,
                captures: 0,
                attempts: 0
            };
        }
        return this.#metrics[key];
    }

    /**
     * @param {StageLocation} location
     * @returns {MetricsEntry?} 
     */
    getLocationStats(location) {
        var key = location.key;
        if (this.#metrics[key]) {
            return Object.assign({}, this.#metrics[key]);
        } else {
            return null;
        }
    }

    *[Symbol.iterator]() {
        for (let entry of Object.values(this.#metrics)) {
            yield Object.assign({}, entry);
        }
    }
}

export class CurrentMetricsDisplay {
    /** @type {HTMLDivElement} */
    #container;

    /** @type {HTMLDivElement} */
    #curCapRateElem;

    constructor() {
        this.#container = createElementWithClasses("div", "current-metrics-container");
        this.#curCapRateElem = createElementWithClasses("div", "current-cap-rate");

        this.#container.replaceChildren(
            createElementWithClasses("h3", "", "Current Section History:"),
            this.#curCapRateElem
        );

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
        this.#container.style.display = (value ? null : "none");
    }

    /**
     * 
     * @param {Metrics} metrics 
     * @param {Game} currentGame 
     */
    update(metrics, currentGame) {
        let curLoc = currentGame.currentLocation;
        if (curLoc.is_unknown) {
            this.shown = false;
            return;
        }

        let curMetrics = metrics.getLocationStats(curLoc);
        if (curMetrics && curMetrics.attempts > 0) {
            this.#curCapRateElem.innerText = formatCapRate(curMetrics.captures, curMetrics.attempts);
            this.shown = true;
        } else {
            this.shown = false;
        }
    }
}

export class MetricsDisplay {
    /** @type {Metrics?} */
    #curMetrics = null;

    /** @type {Game?} */
    #curGame = null;

    /** @type {HTMLElement} */
    #container;

    /** @type {HTMLElement} */
    #listElem;

    /** @type {CurrentMetricsDisplay} */
    #curMetricsDisplay;

    /**
     * 
     * @param {HTMLElement} container 
     */
    constructor(container) {
        this.#container = container;
        this.#listElem = createElementWithClasses("div", "metrics-list");
        this.#curMetricsDisplay = new CurrentMetricsDisplay();

        this.#container.replaceChildren(
            createElementWithClasses("h2", "metrics-header", "Session Metrics"),
            this.#listElem,
            this.#curMetricsDisplay.rootElement
        );
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
        this.#container.style.display = (value ? null : "none");
    }

    /**
     * @param {Game[]} games 
     */
    updateMetrics(games) {
        this.#curMetrics = new Metrics(games);

        Element.prototype.replaceChildren.apply(
            this.#listElem,
            Array.from(this.#curMetrics).filter(
                (entry) => !entry.location.is_unknown
            ).sort(
                (a, b) => {
                    let failuresA = a.attempts - a.captures;
                    let failuresB = b.attempts - b.captures;

                    if (isNaN(failuresA) || failuresA < 0) failuresA = 0;
                    if (isNaN(failuresB) || failuresB < 0) failuresB = 0;

                    return failuresB - failuresA;
                }
            ).map(
                (entry) => createElementWithClasses("div", "metrics-entry", entry.location + ": " + formatCapRate(entry.captures, entry.attempts))
            )
        );

        if (this.#curGame) {
            this.#curMetricsDisplay.update(this.#curMetrics, this.#curGame);
        } else {
            this.#curMetricsDisplay.shown = false;
        }
    }

    /**
     * 
     * @param {Game} curGame 
     */
    updateCurrentGame(curGame) {
        this.#curGame = curGame;

        if (this.#curMetrics) {
            this.#curMetricsDisplay.update(this.#curMetrics, this.#curGame);
        } else {
            this.#curMetricsDisplay.shown = false;
        }
    } 
}
