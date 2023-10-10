import { PracticeAttempts, SetMetrics } from "./game_data/practice_set.js";
import { PracticeDisplay } from "./set_display.js";
import { StatusDisplay } from "./status_display.js";
import { GameId } from "./game_data/named_value.js";

const { invoke, event } = window.__TAURI__;

export class Main {
    /** @type {Main} */
    static #inst;

    /** @type {boolean} */
    static #initializing = false;

    #unregister = [];

    /** @type {PracticeDisplay} */
    #practiceDisplay;

    /** @type {StatusDisplay} */
    #statusDisplay;

    /**  @type {number}  */
    #updateInterval;

    constructor () {
        if (!Main.#initializing) {
            throw TypeError("Instances of Main cannot be constructed directly; use Main.instance instead");
        }
        Main.#initializing = false;

        this.#practiceDisplay = new PracticeDisplay();
        this.#statusDisplay = new StatusDisplay();

        document.getElementById("content-container").replaceChildren(
            this.#practiceDisplay.rootElement,
            this.#statusDisplay.rootElement
        );

        this.#registerEventHandler("error", (ev) => {
            console.error(ev.payload);
        });

        this.#registerEventHandler("attached", (ev) => {
            console.log("Attached to PID " + ev.payload.pid);
            this.#statusDisplay.currentGame = GameId.deserialize(ev.payload.game);
        });

        this.#registerEventHandler("detached", (ev) => {
            console.log("Detached from PID " + ev.payload.pid);
            this.#statusDisplay.currentGame = null;
        });

        this.#registerEventHandler("updated", (ev) => {
            this.#updateGame(ev.payload);
        });

        this.#updateInterval = setInterval(() => {
            this.#updateGame(null);
        }, 100);
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
                }
            })
        );
    }

    /**
     * 
     * @param {number?} gameId 
     */
    #updateGame(gameId) {
        invoke('get_practice_data', { game_id: gameId }).then((setInfo) => {
            /** @type {PracticeAttempts[]} */
            let attempts = setInfo.map(PracticeAttempts.deserialize);

            Main.#inst.#practiceDisplay.update(attempts);
            Main.#inst.#statusDisplay.overallSetMetrics = SetMetrics.total(attempts.map((loc) => loc.totalMetrics()));
            Main.#inst.#statusDisplay.game = gameId;
        }).catch(console.error);
    }

    static {
        document.addEventListener('DOMContentLoaded', () => {
            Main.#initializing = true;
            Main.#inst = new Main();
            invoke('start_watcher', {});
        });
    }
}
