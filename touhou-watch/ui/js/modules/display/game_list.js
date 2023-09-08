import Game from '../game.js';
import GameDisplay from './game_display.js';
import Display from './display.js';
import { newElement } from '../utils.js';

class FinishedGameDisplay extends Display {
    static #HEADER_CELLS = [
        ["index", "#"],
        ["character", "Character"],
        ["mode", "Mode"],
        ["location", "Location"],
        ["result", "Result"],
        ["location-count", "Locs Seen"],
        ["duration", "Time"]
    ];

    /** @type {GameDisplay[]} */
    #displays = [];

    /** @type {HTMLDivElement} */
    #headerElem;

    constructor() {
        super(newElement("div", { className: "game-list" }));

        var headerCells = FinishedGameDisplay.#HEADER_CELLS.map(
            (pair) => newElement("div", {
                className: "game-details-entry game-details-" + pair[0],
                text: pair[1]
            })
        );

        var listHeader = newElement("div", {
            className: "game-summary-row game-details-row game-list-header",
            children: headerCells
        });

        this.#headerElem = newElement("div", {
            className: "game-container game-header-container",
            children: [listHeader]
        });

        this.update();
    }

    /** @returns {Game[]} */
    get games() {
        return this.#displays.map((display) => display.game);
    }

    /** @returns {number} */
    get nGames() {
        return this.games.length;
    }

    /** @returns {HTMLElement[]} */
    get #displayElems() {
        return this.#displays.map((display) => display.rootElement);
    }

    /**
     * 
     * @param {Game | GameDisplay} gameOrDisplay 
     */
    addGame(gameOrDisplay) {
        var gameNumber = this.#displays.length + 1;

        /** @type {GameDisplay} */
        var display = null;

        if (gameOrDisplay instanceof Game) {
            display = new GameDisplay(gameNumber, gameOrDisplay);
        } else if (gameOrDisplay instanceof GameDisplay) {
            display = gameOrDisplay;
            display.gameNumber = gameNumber;
        } else {
            throw new TypeError("gameOrDisplay must be either a Game or a GameDisplay");
        }

        this.#displays.push(display);
        this.update();
    }

    update() {
        this.#displays.sort((a, b) => a.game.startTime.valueOf() - b.game.startTime.valueOf()).forEach((display, idx) => display.gameNumber = idx + 1);
        Element.prototype.replaceChildren.apply(this.rootElement, [this.#headerElem].concat(this.#displayElems));

        if (this.#displays.length > 0) {
            this.#displays[this.#displays.length - 1].scrollIntoView();
        }
    }
}

export class GameListDisplay extends Display {
    /** @type {FinishedGameDisplay} */
    #practices;

    /** @type {FinishedGameDisplay} */
    #fullRuns;

    /** @type {'full' | 'practice'} */
    #shownScreen = 'full';

    /** @type {{ [key: string]: HTMLElement }} */
    #selectorElems = [];

    /** @type {GameDisplay?} */
    #currentDisplay = null;

    constructor() {
        super(newElement("div", { className: "game-list-container", children: [] }));

        this.#selectorElems = {
            'full': newElement("div", { className: "game-selector", text: "Full Runs" }),
            'practice': newElement("div", { className: "game-selector", text: "Practices" })
        };

        this.#practices = new FinishedGameDisplay();
        this.#fullRuns = new FinishedGameDisplay();

        this.#practices.shown = false;
        this.#fullRuns.shown = false;

        var selectorRoot = newElement("div", {
            className: "game-selector-container",
            children: [this.#selectorElems.full, this.#selectorElems.practice]
        });

        Object.entries(this.#selectorElems).forEach((pair) => {
            pair[1].addEventListener('click', (ev) => {
                this.shownScreen = pair[0];
            });
        });

        this.rootElement.replaceChildren(
            selectorRoot,
            this.#fullRuns.rootElement,
            this.#practices.rootElement
        );

        this.update();
    }

    /** @type {Game?} */
    get currentGame() {
        if (this.#currentDisplay) {
            return this.#currentDisplay.game;
        } else {
            return null;
        }
    }

    /** @returns {Game[]} */
    get practices() {
        return this.#practices.games;
    }

    /** @returns {Game[]} */
    get fullRuns() {
        return this.#fullRuns.games;
    }

    /** @returns {Game[]} */
    get finishedGames() {
        return this.practices.concat(this.fullRuns);
    }

    /** @returns {Game[]} */
    get allGames() {
        var ret = this.finishedGames;
        if (this.#currentDisplay) {
            ret.push(this.#currentDisplay.game);
        }
        return ret;
    }

    /** @returns {'full' | 'practice'} */
    get shownScreen() {
        return this.#shownScreen;
    }

    /** @param {'full' | 'practice'} value */
    set shownScreen(value) {
        this.#shownScreen = value;
        this.update();
    }

    /** @returns {boolean} */
    #updateCurrentGameTime() {
        if (this.#currentDisplay && !this.#currentDisplay.game.ended) {
            this.#currentDisplay.updateTime();
            return true;
        } else {
            return false;
        }
    }

    #flushCurrentGame() {
        if (this.#currentDisplay) {
            this.#currentDisplay.update();
            this.#currentDisplay.rootElement.classList.remove("current-game");
            this.endAnimation('update-current-time');

            if (this.#currentDisplay.game.practice) {
                this.#practices.addGame(this.#currentDisplay);
            } else {
                this.#fullRuns.addGame(this.#currentDisplay);
            }

            this.shownScreen = this.#currentDisplay.game.practice ? 'practice' : 'full';
        }

        this.#currentDisplay = null;
    }

    /**
     * 
     * @param {Game} newData 
     */
    updateCurrentGame(newData) {
        if (this.#currentDisplay) {
            this.#currentDisplay.game = newData;
        } else {
            let gameNumber = (newData.practice ? this.#practices.nGames : this.#fullRuns.nGames) + 1;

            this.#currentDisplay = new GameDisplay(gameNumber, newData);
            this.#currentDisplay.rootElement.classList.add("current-game");
            this.rootElement.appendChild(this.#currentDisplay.rootElement);
            this.#currentDisplay.scrollIntoView();
            this.startAnimation('update-current-time', (ts) => this.#updateCurrentGameTime());

            this.shownScreen = newData.practice ? 'practice' : 'full';
        }

        if (newData.ended) {
            this.#flushCurrentGame();
        }
    }

    forceEndCurrentGame() {
        if (this.#currentDisplay) {
            this.#currentDisplay.game.forceEnd();
            this.#flushCurrentGame();
        }
    }

    update() {
        Object.entries(this.#selectorElems).forEach((pair) => {
            if (this.#shownScreen === pair[0]) {
                pair[1].classList.add('selected');
            } else {
                pair[1].classList.remove('selected');
            }
        });

        if (this.#shownScreen === 'full') {
            this.#fullRuns.shown = true;
            this.#practices.shown = false;
        } else if (this.#shownScreen === 'practice') {
            this.#fullRuns.shown = false;
            this.#practices.shown = true;
        }
    }
}
