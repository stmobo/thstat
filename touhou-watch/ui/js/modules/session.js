import Game from './game.js';
import { EndGameEvent, GameEvent, StartGameEvent } from './game_data/game_event.js';
import GameDisplay from './game_display.js';

/** @typedef {[Game, GameDisplay]} DisplayedGame */

export default class Session {
    /** @type {DisplayedGame[]} */
    #endedGames = [];

    /** @type {DisplayedGame?} */
    #currentGame = null;

    /** @type {HTMLElement} */
    #listElem;

    /** @type {HTMLElement} */
    #listHeaderContainer;

    constructor (listElem, headerContainerElem) {
        this.#listElem = listElem;
        this.#listHeaderContainer = headerContainerElem;
        requestAnimationFrame(() => this.#refreshGameTime());
    }

    /** @returns {Game?} */
    get currentGame() {
        return this.#currentGame ? this.#currentGame[0] : null;
    }

    /** @returns {Game[]} */
    get endedGames() {
        return this.#endedGames.map((pair) => pair[0]);
    }

    /** @returns {Game[]} */
    get allGames() {
        let ret = this.endedGames;
        if (this.currentGame) {
            ret.push(this.currentGame);
        }
        return ret;
    }

    #refreshGameTime() {
        if (this.#currentGame && !this.#currentGame[0].ended) {
            this.#currentGame[1].updateTime();
        }

        requestAnimationFrame(() => this.#refreshGameTime());
    }

    #updateListLayout() {
        this.#endedGames.sort((a, b) => a[0].startTime.valueOf() - b[0].startTime.valueOf());
        this.#endedGames.forEach((pair, idx) => {
            pair[1].gameNumber = idx + 1;
        });

        this.#listElem.replaceChildren(this.#listHeaderContainer);
        Element.prototype.append.apply(this.#listElem, this.#endedGames.map((pair) => pair[1].rootElement));

        if (this.#currentGame) {
            this.#currentGame[1].gameNumber = this.#endedGames.length + 1;
            this.#listElem.append(this.#currentGame[1].rootElement);
        }
    }

    #flushCurrentGame() {
        if (this.#currentGame) {
            this.#currentGame[1].update();
            this.#endedGames.push(this.#currentGame);
            this.#currentGame = null;
        }
    }

    forceEndCurrentGame() {
        if (this.#currentGame) {
            this.#currentGame[0].forceEnd();
            this.#flushCurrentGame();
            this.#updateListLayout();
        }
    }

    /**
     * 
     * @param {GameEvent[]} events 
     */
    addEvents(events) {
        var updateCurrentDisplay = false;
        var requiresLayoutUpdate = false;

        for (let event of events) {
            if (event instanceof StartGameEvent) {
                if (this.#currentGame) {
                    this.#flushCurrentGame();
                }

                let newGame = new Game(event);
                let newDisplay = new GameDisplay(this.#endedGames.length + 1, newGame);
                this.#currentGame = [newGame, newDisplay];
                updateCurrentDisplay = false;
                requiresLayoutUpdate = true;
            } else if (this.#currentGame) {
                this.#currentGame[0].addEvent(event);
                updateCurrentDisplay = true;

                if (event instanceof EndGameEvent) {
                    this.#flushCurrentGame();
                    updateCurrentDisplay = false;
                    requiresLayoutUpdate = true;
                }
            } else {
                console.log("Ignoring event with no running game: ", event);
            }
        }

        if (updateCurrentDisplay && this.#currentGame) {
            this.#currentGame[1].update();
        }

        if (requiresLayoutUpdate) {
            this.#updateListLayout();
        }
    }
}
