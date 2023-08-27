import { GameEvent, StartGameEvent } from "./game_data/game_event.js";
import Game from "./game.js";

export default class LogDisplay {
    /** @type {number} */
    static #MAX_HISTORY = 250;

    /** @type {HTMLElement} */
    #container;

    /**
     * 
     * @param {HTMLElement} container 
     */
    constructor (container) {
        this.#container = container;
    }

    /**
     * 
     * @param {string} time 
     * @param {string} text 
     * @param {string} extraClasses
     */
    #addLine(time, text, extraClasses = "") {
        var row = document.createElement("div");
        var timeSpan = document.createElement("span");
        var textSpan = document.createElement("span");

        timeSpan.className = "event-time";
        timeSpan.innerText = time;
        
        textSpan.className = "event-text " + extraClasses;
        textSpan.innerText = text;

        row.className = "event-row";
        row.replaceChildren(timeSpan, textSpan);

        this.#container.appendChild(row);

        while ((this.#container.childNodes.length > LogDisplay.#MAX_HISTORY) && this.#container.firstChild) {
            this.#container.firstChild.remove();
        }

        this.#container.scrollTop = this.#container.scrollHeight;
    }

    /**
     * Log a text message.
     * 
     * @param {string} text 
     * @param {string} extraClasses
     */
    logMessage(text, extraClasses = "") {
        this.#addLine((new Date()).toISOString(), text, extraClasses);
    }

    /**
     * Log a game event.
     * 
     * @param {GameEvent} event
     * @param {Game?} currentGame
     */
    logGameEvent(event, currentGame = null) {
        if (currentGame && !currentGame.ended && !(event instanceof StartGameEvent)) {
            let elapsedSecs = (event.time.valueOf() - currentGame.startTime.valueOf()) / 1000.0;
            this.#addLine(
                (elapsedSecs >= 0 ? "+" : "") + elapsedSecs.toFixed(3),
                event.toString()
            );
        } else {
            this.#addLine(event.time.toISOString(), event.toString());
        }
    }
}