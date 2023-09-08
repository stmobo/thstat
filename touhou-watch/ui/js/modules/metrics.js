import Game from "./game.js";
import { BombEvent, EndGameEvent, EnterSectionEvent, BorderEndEvent, GameEvent, MissEvent, StartGameEvent } from "./game_data/game_event.js";
import { StageLocation } from "./game_data/locations.js";
import { formatDuration } from './utils.js';

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

export class SectionEvents {
    static #internalOnly = false;

    /** 
     * @type {GameEvent[]}
     */
    #events;

    /**
     * 
     * @param {GameEvent[]} events 
     */
    constructor (events) {
        if (!SectionEvents.#internalOnly) {
            throw new TypeError("SectionEvents instances cannot be constructed directly, use SectionEvents.fromEvents instead");
        }
        SectionEvents.#internalOnly = false;

        let sorted = events.slice();
        if (sorted.length < 2) {
            throw new Error("events array does not have enough elements");
        }

        let startLocation = sorted[0].location;
        if (!startLocation || sorted.some((ev, idx) => (idx < (sorted.length - 1)) && !!ev.location && !ev.location.equals(startLocation))) {
            throw new Error("events array has non-matching locations");
        }

        this.#events = sorted;
    }

    /**
     * 
     * @param {GameEvent[]} events 
     * @returns {SectionEvents[]}
     */
    static fromEvents(events) {
        let sorted = events.slice().sort((a, b) => a.time.valueOf() - b.time.valueOf());
        let curIdx = 0;
        let curLocation = sorted[0].location;

        if (!curLocation) {
            throw new Error("First event has no listed location");
        }

        let ret = [];
        while (curIdx < (sorted.length - 1)) {
            let nextIdx = sorted.findIndex((ev, idx) => (idx > curIdx) && !!ev.location && !ev.location.equals(curLocation));
            if (nextIdx < 0) nextIdx = sorted.length - 1;

            if (nextIdx - curIdx >= 1) {
                SectionEvents.#internalOnly = true;
                ret.push(new SectionEvents(sorted.slice(curIdx, nextIdx + 1)));
            }

            curIdx = nextIdx;
            curLocation = sorted[curIdx].location;
        }

        return ret;
    }

    /** @returns {GameEvent} */
    get startEvent() {
        return this.#events[0];
    }

    /** @returns {GameEvent} */
    get endEvent() {
        return this.#events[this.#events.length - 1];
    }

    /** @returns {GameEvent[]} */
    get duringEvents() {
        return this.#events.slice(1, this.#events.length - 1);
    }

    /** @returns {StageLocation} */
    get location() {
        return this.startEvent.location;
    }

    /** @returns {StageLocation} */
    get endLocation() {
        return this.endEvent.location;
    }
    
    /** @returns {number} */
    get duration() {
        return this.endEvent.time.valueOf() - this.startEvent.time.valueOf();
    }

    /** @returns {boolean} */
    get captured() {
        var lastEvent = this.#events[this.#events.length - 1];
        return !(
            this.#events.slice(0, -1).some((ev) => (ev instanceof BombEvent) || (ev instanceof MissEvent) || ((ev instanceof BorderEndEvent) && ev.broken))
            || ((lastEvent instanceof EndGameEvent) && !lastEvent.cleared)
        );
    }

    /** @returns {number} */
    get bombs() {
        return this.#events.slice(0, -1).reduce((acc, ev) =>  (ev instanceof BombEvent) ? acc + 1 : acc, 0);
    }

    /** @returns {number} */
    get breaks() {
        return this.#events.slice(0, -1).reduce((acc, ev) => ((ev instanceof BorderEndEvent) && ev.broken) ? acc + 1 : acc, 0);
    }

    /** @returns {number} */
    get misses() {
        return this.#events.slice(0, -1).reduce((acc, ev) => (ev instanceof MissEvent) ? acc + 1 : acc, 0);
    }
}

export class RunLife {
    static #internalOnly = false;

    /** @type {StartGameEvent | MissEvent} */
    #startEvent;

    /** @type {EndGameEvent | MissEvent} */
    #endEvent;

    /** @type {SectionEvents[]} */
    #sections = [];

    /**
     * 
     * @param {StartGameEvent | MissEvent} startEvent 
     * @param {EndGameEvent | MissEvent} endEvent
     * @param {SectionEvents[]} sections
     */
    constructor (startEvent, endEvent, sections) {
        if (!RunLife.#internalOnly) {
            throw new TypeError("RunLife instances cannot be constructed directly");
        }
        RunLife.#internalOnly = false;

        this.#startEvent = startEvent;
        this.#endEvent = endEvent;
        this.#sections = sections;
    }

    /**
     * 
     * @param {GameEvent[]} events 
     * @returns {RunLife[]}
     */
    static fromEvents(events) {
        let sorted = events.slice().sort((a, b) => {
            if ((a instanceof StartGameEvent) && !(b instanceof StartGameEvent)) {
                return -1;
            } else if ((a instanceof EndGameEvent) && !(b instanceof EndGameEvent)) {
                return 1;
            } else {
                return a.time.valueOf() - b.time.valueOf();
            }
        });

        if (sorted.length < 2) {
            throw new Error("events array does not have enough events");
        }

        let curStartIdx = 0;
        let ret = [];
        while (curStartIdx < (sorted.length - 1)) {
            let nextIdx = sorted.findIndex((ev, idx) => (idx > curStartIdx) && (ev instanceof MissEvent));
            if (nextIdx < 0) nextIdx = sorted.length - 1;

            if (nextIdx - curStartIdx >= 1) {
                RunLife.#internalOnly = true;
                let sections = SectionEvents.fromEvents(sorted.slice(curStartIdx, nextIdx + 1))
                ret.push(new RunLife(sorted[curStartIdx], sorted[nextIdx], sections));
            }

            curStartIdx = nextIdx;
        }

        return ret;
    }

    /** @returns {StartGameEvent | MissEvent} */
    get startEvent() {
        return this.#startEvent;
    }

    /** @returns {EndGameEvent | MissEvent} */
    get endEvent() {
        return this.#endEvent;
    }

    /** @returns {SectionEvents[]} */
    get sections() {
        return this.#sections;
    }

    /** @returns {number} */
    get duration() {
        return this.#endEvent.time.valueOf() - this.#startEvent.time.valueOf();
    }
}

export class MetricsEntry {
    /** @type {SectionEvents[]} */
    #attemptEvents;

    /**
     * 
     * @param {SectionEvents[]} attemptEvents 
     */
    constructor (attemptEvents) {
        if (attemptEvents.length === 0) {
            throw new Error("attemptEvents array is empty");
        }

        var location = attemptEvents[0].location;
        this.#attemptEvents = attemptEvents;

        if (attemptEvents.some((evts) => !evts.location.equals(location))) {
            throw new Error("attemptEvents array has elements with non-matching locations");
        }
    }

    /** @returns {StageLocation} */
    get location() {
        return this.#attemptEvents[0].location;
    }

    /** @returns {number} */
    get misses() {
        return this.#attemptEvents.reduce((acc, val) => acc + val.misses, 0);
    }

    /** @returns {number} */
    get bombs() {
        return this.#attemptEvents.reduce((acc, val) => acc + val.bombs, 0);
    }

    /** @returns {number} */
    get breaks() {
        return this.#attemptEvents.reduce((acc, val) => acc + val.breaks, 0);
    }

    /** @returns {number} */
    get captures() {
        return this.#attemptEvents.reduce((acc, val) => acc + val.captured, 0);
    }

    /** @returns {number} */
    get attempts() {
        return this.#attemptEvents.length;
    }

    /** @returns {number[]} */
    get durations() {
        return this.#attemptEvents.map((val) => val.duration);
    }
}

export class Metrics {
    /** @type {{ [shot: number]: {[key: string]: MetricsEntry} }} */
    #metrics;

    /**
     * @param {Game[]} games 
     */
    constructor(games) {
        var sectionAttempts = {};

        for (let game of games) {
            if (game.events.length < 2) continue;

            let shot = game.shot.id;
            if (!sectionAttempts[shot]) sectionAttempts[shot] = {};

            for (let section of SectionEvents.fromEvents(game.events)) {
                if (!sectionAttempts[shot][section.location.key]) {
                    sectionAttempts[shot][section.location.key] = [section]; 
                } else {
                    sectionAttempts[shot][section.location.key].push(section);
                }
            }
        }
        
        this.#metrics = {};
        for (let pair of Object.entries(sectionAttempts)) {
            let id = pair[0];
            this.#metrics[id] = {};
            for (let pair2 of Object.entries(pair[1])) {
                this.#metrics[id][pair2[0]] = new MetricsEntry(pair2[1]);
            }
        }
    }

    /**
     * @param {ShotType} shot
     * @param {StageLocation} location
     * @returns {MetricsEntry?} 
     */
    getLocationStats(shot, location) {
        if (!this.#metrics[shot.id]) return null;
        return this.#metrics[shot.id][location.key];
    }

    *[Symbol.iterator]() {
        for (let entry of Object.values(this.#metrics)) {
            for (let subentry of Object.values(entry)) {
                yield subentry;
            }
        }
    }
}

export class CurrentMetricsDisplay {
    /** @type {HTMLDivElement} */
    #container;

    /** @type {HTMLDivElement} */
    #curCapRateElem;

    /** @type {HTMLDivElement} */
    #curLifetimeElem;

    /** @type {HTMLDivElement} */
    #curMissElem;

    /** @type {HTMLDivElement} */
    #curBombElem;

    /** @type {HTMLDivElement} */
    #curBreakElem;

    constructor() {
        this.#container = createElementWithClasses("div", "current-metrics-container");
        this.#curCapRateElem = createElementWithClasses("div", "current-metric current-cap-rate");
        this.#curLifetimeElem = createElementWithClasses("div", "current-metric current-avg-lifetime");
        this.#curMissElem = createElementWithClasses("div", "current-metric current-miss-count");
        this.#curBombElem = createElementWithClasses("div", "current-metric current-bomb-count");
        this.#curBreakElem = createElementWithClasses("div", "current-metric current-break-count");

        this.#container.replaceChildren(
            createElementWithClasses("h3", "", "Current Section History:"),
            this.#curCapRateElem,
            this.#curLifetimeElem,
            this.#curMissElem,
            this.#curBombElem,
            this.#curBreakElem
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
        let curMetrics = metrics.getLocationStats(currentGame.shot, curLoc);

        if (curMetrics && curMetrics.attempts > 0) {
            let lifetimes = curMetrics.durations;
            let avgLifetime =  (lifetimes.length > 0) ? Math.floor(lifetimes.reduce((acc, val) => acc + val, 0) / lifetimes.length) : 0;

            this.#curCapRateElem.innerText = formatCapRate(curMetrics.captures, curMetrics.attempts);
            this.#curLifetimeElem.innerText = formatDuration(avgLifetime);
            this.#curMissElem.innerText = curMetrics.misses;
            this.#curBombElem.innerText = curMetrics.bombs;
            this.#curBreakElem.innerText = curMetrics.breaks;

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

    /** @type {HTMLElement} */
    #gameRuntimeElem;

    /** @type {Date?} */
    #gameStartTime = null;

    /** @type {number?} */
    #runtimeAnimationId = null;

    /**
     * 
     * @param {HTMLElement} container 
     */
    constructor(container) {
        this.#container = container;
        this.#listElem = createElementWithClasses("div", "metrics-list");
        this.#gameRuntimeElem = createElementWithClasses("div", "metrics-game-runtime");
        this.#curMetricsDisplay = new CurrentMetricsDisplay();

        this.#container.replaceChildren(
            createElementWithClasses("h2", "metrics-header", "Session Metrics"),
            this.#listElem,
            this.#curMetricsDisplay.rootElement,
            this.#gameRuntimeElem
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

    /** @returns {boolean} */
    get gameRunning() {
        return !!this.#gameStartTime;
    }

    /** @param {boolean} active */
    set gameRunning(active) {
        if (active) {
            this.#gameStartTime = new Date();
            this.#runtimeAnimationId = requestAnimationFrame((_) => this.#updateGameRuntime());
        } else {
            this.#gameStartTime = null;
            this.#gameRuntimeElem.style.display = "none";

            if (this.#runtimeAnimationId) {
                cancelAnimationFrame(this.#runtimeAnimationId);
            }

            this.#runtimeAnimationId = null;
        }
    }

    #updateGameRuntime() {
        if (this.#gameStartTime) {
            this.#gameRuntimeElem.innerText = formatDuration(Date.now() - this.#gameStartTime.valueOf(), 0);
            this.#gameRuntimeElem.style.display = null;
            this.#runtimeAnimationId = requestAnimationFrame((_) => this.#updateGameRuntime());
        } else {
            this.#gameRuntimeElem.style.display = "none";
            this.#runtimeAnimationId = null;
        }
    }

    /**
     * @param {Game[]} games 
     */
    updateMetrics(games) {
        this.#curMetrics = new Metrics(games);

        Element.prototype.replaceChildren.apply(
            this.#listElem,
            Array.from(this.#curMetrics).sort(
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
