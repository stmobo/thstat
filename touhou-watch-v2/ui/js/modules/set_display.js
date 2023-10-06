import Display from "./display.js";
import { PracticeAttempts, SetMetrics } from "./game_data/practice_set.js";
import { newElement, formatPercent, formatDuration } from "./utils.js";


class PracticeLocationDisplay extends Display {
    static #ROW_CELLS = 5;

    /** @type {HTMLDivElement} */
    #gameElem;

    /** @type {HTMLDivElement} */
    #locationElem;

    /** @type {HTMLDivElement} */
    #difficultyElem;

    /** @type {HTMLDivElement} */
    #shotElem;

    /** @type {HTMLDivElement} */
    #attemptsElem;

    /** @type {HTMLDivElement} */
    #capRateElem;
    
    /** @type {HTMLDivElement} */
    #timeElem;

    /** @type {PracticeAttempts} */
    #practiceSets;

    /** @type {HTMLDivElement[]} */
    #setElems;

    /**
     * 
     * @param {PracticeAttempts} practiceSets 
     */
    constructor(practiceSets) {
        super(newElement("div", { className: "practice-location-display " + practiceSets.game.cssClass }));

        this.#gameElem = newElement("div", { "className": "practice-header-elem practice-set-game" });
        this.#locationElem = newElement("div", { "className": "practice-header-elem practice-set-location" });
        this.#difficultyElem = newElement("div", { "className": "practice-header-elem practice-set-difficulty" });
        this.#shotElem = newElement("div", { "className": "practice-header-elem practice-set-shot" });
        this.#attemptsElem = newElement("div", { "className": "practice-header-elem practice-set-total-attempts" });
        this.#capRateElem = newElement("div", { "className": "practice-header-elem practice-set-cap-rate" });
        this.#timeElem = newElement("div", { "className": "practice-header-elem practice-set-time" });

        this.rootElement.replaceChildren(
            this.#gameElem,
            this.#locationElem,
            this.#difficultyElem,
            this.#shotElem,
            this.#attemptsElem,
            this.#capRateElem,
            this.#timeElem,
            newElement("div", { "className": "practice-cells-separator", text: "Sets:" })
        );

        this.#practiceSets = practiceSets;
        this.#gameElem.innerText = this.#practiceSets.game.abbreviation;
        this.#locationElem.innerText = this.#practiceSets.location.name;
        this.#difficultyElem.innerText = this.#practiceSets.difficulty.name;
        this.#shotElem.innerText = this.#practiceSets.shotType.name;
        
        this.#setElems = [];
        this.update(practiceSets);
    }

    /** @returns {PracticeAttempts} */
    get practiceSets() {
        return this.#practiceSets;
    }

    /**
     * @param {SetMetrics[]} values
     * @returns {HTMLDivElement[]}
     */
    static #setsToElements(values) {
        if (values.length > 0) {
            let vals = values.map((val) => val.captures + "/" + val.attempts);
            while (vals.length % PracticeLocationDisplay.#ROW_CELLS != 0) {
                vals.push("");
            }

            return vals.map((val, idx) => {
                return newElement("div", {
                    className: "set-cell" + (idx % PracticeLocationDisplay.#ROW_CELLS == 0 ? " set-cell-row-end" : ""),
                    text: val
                });
            });
        } else {
            return [];
        }
    }

    /**
     * 
     * @param {PracticeAttempts} practiceSets 
     */
    update(practiceSets) {
        if (practiceSets.key != this.#practiceSets.key) {
            throw new Error("non-matching set key (got " + practiceSets.key + ", expected " + this.#practiceSets.key + ")");
        }

        this.#practiceSets = practiceSets;

        let sets = this.#practiceSets.setMetrics();
        let caps_attempts = SetMetrics.total(sets);

        this.#attemptsElem.innerText = caps_attempts.captures + " / " + caps_attempts.attempts;
        this.#capRateElem.innerText = formatPercent(caps_attempts.captures / caps_attempts.attempts, 1);
        this.#timeElem.innerText = formatDuration(caps_attempts.totalTime, 1);

        this.#setElems.forEach((elem) => elem.remove());
        this.#setElems = PracticeLocationDisplay.#setsToElements(sets);

        Element.prototype.append.apply(this.rootElement, this.#setElems);
    }
}

export class PracticeDisplay extends Display {
    /** @type { {[key: string]: PracticeLocationDisplay} } */
    #locations;

    constructor() {
        super(newElement("div", { className: "practice-display" }));
        this.#locations = {};
    }

    /** @param {PracticeAttempts[]} updateLocations */
    update(updateLocations) {
        for (let practiceSets of updateLocations) {
            let key = practiceSets.key;
            if (this.#locations[key]) {
                this.#locations[key].update(practiceSets);
            } else {
                this.#locations[key] = new PracticeLocationDisplay(practiceSets);
            }
        }

        var displays = Object.values(this.#locations).sort(
            (a, b) => b.practiceSets.lastAtempt().valueOf() - a.practiceSets.lastAtempt().valueOf()
        ).map((disp) => disp.rootElement);

        Element.prototype.replaceChildren.apply(this.rootElement, displays);
    }
}
