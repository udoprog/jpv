// Ping interval.
const INTERVAL = 1000;
// Deadline until we considered a window dead.
const DEADLINE = 5000;

export class Pinger {
    #onTimeout: () => void;
    #onPing: (payload: string) => void;
    #deadline: number | null;
    #lastPing: string | null;
    #sequence: number;
    #interval: number | null;

    constructor(onTimeout: () => void, onPing: (payload: string) => void) {
        this.#onTimeout = onTimeout;
        this.#onPing = onPing;
        this.#deadline = null;
        this.#lastPing = null;
        this.#sequence = 0;
        this.#interval = null;
    }

    start() {
        if (this.#interval === null) {
            this.#interval = setInterval(this.#onInterval.bind(this), INTERVAL);
        }
    }

    stop() {
        if (this.#interval !== null) {
            clearInterval(this.#interval);
            this.#interval = null;
        }

        if (this.#deadline !== null) {
            clearTimeout(this.#deadline);
            this.#deadline = null;
        }

        this.#lastPing = null;
    }

    restart() {
        this.stop();
        this.start();
    }

    receivePong(payload: string) {
        if (this.#lastPing !== payload) {
            return;
        }

        this.#lastPing = null;

        if (this.#deadline !== null) {
            clearTimeout(this.#deadline);
            this.#deadline = null;
        }
    }

    #timeout() {
        this.#onTimeout();
        this.#deadline = null;
    }

    #onInterval() {
        if (this.#lastPing === null) {
            this.#lastPing = `ping${this.#sequence}`;
            this.#sequence += 1;
            this.#sequence %= 1000000;
        }

        if (this.#deadline === null) {
            this.#deadline = setTimeout(this.#timeout.bind(this), DEADLINE);
        }

        this.#onPing(this.#lastPing);
    }
}
