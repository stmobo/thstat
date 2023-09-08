import LogDisplay from './log_display.js';
import { MetricsDisplay } from './metrics.js';
import SpellCard from './game_data/spell_card.js';
import Game from './game.js';
import { GameEvent } from './game_data/game_event.js';
import { GameListDisplay } from './display/game_list.js';

const { invoke, event } = window.__TAURI__;

export class Main {
    /** @type {Main} */
    static #inst;

    /** @type {boolean} */
    static #initializing = false;

    #unregister = [];

    /** @type {LogDisplay} */
    #logDisplay;

    /** @type {GameListDisplay} */
    #gameList;

    /** @type {MetricsDisplay} */
    #metricsDisplay;

    /** @type {boolean} */
    static #gameAttached;

    constructor () {
        if (!Main.#initializing) {
            throw TypeError("Instances of Main cannot be constructed directly; use Main.instance instead");
        }
        Main.#initializing = false;

        this.#logDisplay = new LogDisplay(document.getElementById("event-log"));
        this.#metricsDisplay = new MetricsDisplay(document.getElementById("metrics-container"));
        this.#gameList = new GameListDisplay();

        document.getElementById("content-container").prepend(this.#gameList.rootElement);

        this.#registerEventHandler("error", (ev) => {
            this.#logDisplay.logMessage(ev.payload, "log-error");
        });

        this.#registerEventHandler("game-attached", (ev) => {
            this.#logDisplay.logMessage("Attached to PID " + ev.payload);
            this.#gameList.forceEndCurrentGame();
            Main.#gameAttached = true;
            this.#metricsDisplay.gameRunning = true;
        });

        this.#registerEventHandler("game-detached", (ev) => {
            this.#logDisplay.logMessage("Waiting for PCB...");
            this.#gameList.forceEndCurrentGame();
            Main.#gameAttached = false;
            this.#metricsDisplay.gameRunning = false;
        });

        this.#registerEventHandler("run-update", (ev) => {
            let finished = ev.payload[0];
            let run = Game.deserialize(ev.payload[1]);
            let events = ev.payload[2].map(GameEvent.deserialize);

            this.#gameList.updateCurrentGame(run);

            for (let ev of events) {
                this.#logDisplay.logGameEvent(ev, this.#gameList.currentGame);
            }

            this.#metricsDisplay.updateMetrics(this.#gameList.finishedGames);
            if (this.#gameList.currentGame) {
                this.#metricsDisplay.updateCurrentGame(this.#gameList.currentGame);
            }
        });
    }

    static get gameAttached() {
        return Main.#gameAttached;
    }

    static get instance() {
        return Main.#inst;
    }

    #registerEventHandler(event_id, handler) {
        this.#unregister.push(
            event.listen(event_id, (ev) => {
                try {
                    handler(ev);
                } catch (e) {
                    console.error(e);
                    this.#logDisplay.logMessage(e.toString(), "log-error");
                }
            })
        );
    }

    /**
     * 
     * @param {string} text 
     * @param {string} extraClasses 
     */
    logMessage(text, extraClasses = "") {
        this.#logDisplay.logMessage(text, extraClasses);
    }

    static {
        document.addEventListener('DOMContentLoaded', () => {
            SpellCard.initData().then(() => {
                Main.#initializing = true;
                Main.#inst = new Main();
                return invoke('init_events', {});
            }).then(() => Main.#inst.logMessage("Started watcher thread"))
        });
    }
}