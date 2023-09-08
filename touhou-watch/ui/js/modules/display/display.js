export default class Display {
    /** @type {HTMLElement} */
    #root;

    /** @type { {[key: string]: { handle: number, callback: (ts: number) => boolean }} } */
    #animCallbacks = {};

    /**
     * 
     * @param {HTMLElement} root 
     */
    constructor (root) {
        this.#root = root;
    }

    /** @returns {HTMLElement} */
    get rootElement() {
        return this.#root;
    }

    /** @returns {boolean} */
    get shown() {
        return this.#root.style.display !== "none";
    }

    /** @param {boolean} value */
    set shown(value) {
        this.#root.style.display = (value ? null : "none");
    }

    /**
     * 
     * @param {string} key 
     * @param {number} ts 
     */
    #updateAnimation(key, ts) {
        if (this.#animCallbacks[key]) {
            if (this.#animCallbacks[key].callback(ts)) {
                this.#animCallbacks[key].handle = requestAnimationFrame(
                    (ts) => this.#updateAnimation(key, ts)
                );
            } else {
                delete this.#animCallbacks[key];
            }
        }
    }

    endAnimation(key) {
        if (this.#animCallbacks[key]) {
            cancelAnimationFrame(this.#animCallbacks[key].handle);
            delete this.#animCallbacks[key];
        }
    }

    /**
     * 
     * @param {string} key 
     * @param {(ts: number) => boolean} callback 
     */
    startAnimation(key, callback) {
        this.endAnimation(key);
        this.#animCallbacks[key] = {
            handle: requestAnimationFrame((ts) => this.#updateAnimation(key, ts)),
            callback: callback
        };
    }

    scrollIntoView() {
        this.rootElement.scrollIntoView(false);
    }

    update() {}
}
