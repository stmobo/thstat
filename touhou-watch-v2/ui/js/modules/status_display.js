import Display from "./display.js";
import { GameId } from "./game_data/named_value.js";
import { newElement, formatDuration, formatPercent } from "./utils.js";
import { PracticeAttempts, SetMetrics } from "./game_data/practice_set.js";
import { TrackingRangeSelector } from "./location_selector.js";

export class StatusDisplay extends Display {
    /** @type {GameId?} */
    #gameAttached = null;

    /** @type {Date?} */
    #attachTime = null;

    /** @type {SetMetrics?} */
    #overallSetMetrics = null;

    /** @type {HTMLDivElement} */
    #attachedElem;

    /** @type {HTMLDivElement} */
    #playtimeElem;

    /** @type {HTMLDivElement} */
    #totalCapsElem;

    /** @type {HTMLDivElement} */
    #overallCapRateElem;

    /** @type {TrackingRangeSelector} */
    #trackRangeSelector;

    constructor () {
        super(newElement("div", { className: "status-display" }));

        this.#attachedElem = newElement("div", { className: "status-elem current-game" });
        this.#playtimeElem = newElement("div", { className: "status-elem current-playtime" });
        this.#totalCapsElem = newElement("div", { className: "status-elem total-captures" });
        this.#overallCapRateElem = newElement("div", { className: "status-elem overall-capture-rate" });
        this.#trackRangeSelector = new TrackingRangeSelector();
        this.#trackRangeSelector.shown = false;
        this.rootElement.replaceChildren(this.#attachedElem, this.#playtimeElem, this.#totalCapsElem, this.#overallCapRateElem, this.#trackRangeSelector.rootElement);

        this.update();
    }

    /** @returns {GameId?} */
    get currentGame() {
        return this.#gameAttached;
    }

    /**
     * @param {GameId?} value
     */
    set currentGame(value) {
        this.#gameAttached = value;
        if (value) {
            this.#attachTime = new Date();
            this.startAnimation("updatePlaytime", () => this.updatePlaytime());
            this.#trackRangeSelector.update(value.value);
            this.#trackRangeSelector.shown = true;
        } else {
            this.#attachTime = null;
            this.endAnimation("updatePlaytime");
            this.#trackRangeSelector.update(null);
            this.#trackRangeSelector.shown = false;
        }

        this.update();
    }

    /** @returns {SetMetrics?} */
    get overallSetMetrics() {
        return this.overallSetMetrics;
    }

    /**
     * @param {SetMetrics?} value
     */
    set overallSetMetrics(value) {
        this.#overallSetMetrics = value;
        this.update();
    }

    /**
     * 
     * @returns {boolean}
     */
    updatePlaytime() {
        if (this.#attachTime) {
            this.#playtimeElem.innerText = formatDuration(Date.now() - this.#attachTime.valueOf());
            this.#playtimeElem.style.display = null;
            return true;
        } else {
            this.#playtimeElem.style.display = "none";
            return false;
        }
    }

    update() {
        this.updatePlaytime();

        this.#totalCapsElem.style.display = "none";
        this.#overallCapRateElem.style.display = "none";
        this.rootElement.classList.remove("attached", "detached", "mof", "pcb", "in");

        if (this.#gameAttached) {
            this.rootElement.classList.add("attached", this.#gameAttached.cssClass);
            this.#attachedElem.innerText = this.#gameAttached.name;
            
            if (this.#overallSetMetrics && this.#overallSetMetrics.attempts > 0) {
                this.#totalCapsElem.style.display = null;
                this.#totalCapsElem.innerText = this.#overallSetMetrics.captures + " / " + this.#overallSetMetrics.attempts;
    
                this.#overallCapRateElem.style.display = null;
                this.#overallCapRateElem.innerText = formatPercent(this.#overallSetMetrics.capRate);
            }
        } else {
            this.rootElement.classList.add("detached");
            this.#attachedElem.innerText = "Waiting for Game...";
        }

    }
}