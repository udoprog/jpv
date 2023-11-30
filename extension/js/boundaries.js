class Boundaries {
    constructor() {
        this._quote = null;
        this._count = 0;
        this._output = [];
        this._whitespace = null;
        this._leading = true;
    }

    /**
     * Populate boundaries from a node.
     *
     * @param {string} content Content to scan.
     * @returns {number[]} Boundaries found.
     */
    populate(node) {
        let content = node.textContent;

        for (let i = 0; i < content.length; i++) {
            this._count += 1;
            let c = content[i];

            if (c === ' ' || c === '　' || c === '\n' || c === '\t') {
                if (this._whitespace === null) {
                    this._whitespace = { node, index: i };
                }

                continue;
            }

            if (this._quote !== null) {
                if (this._quote.expected !== c) {
                    continue;
                }

                this._output.push({ node: this._quote.node, index: this._quote.index });
                this._output.push({ node, index: i + 1 });
                this._quote = null;
            }

            if (c === '「') {
                this._quote = { node, index: i, expected: '」' };
                continue;
            }

            if (c === '\"') {
                this._quote = { node, index: i, expected: '\"' };
                continue;
            }

            if (this._leading && this._whitespace !== null) {
                this._output.push({ node, index: i });
            }

            this._leading = false;
            this._whitespace = null;

            if (isPunct(c)) {
                while (isPunctOrNumerical(content[i + 1])) {
                    i += 1;
                }

                this._output.push({ node, index: i + 1 });
                this._punct = node;
            }
        }
    }

    output() {
        // Populate trailing whitespace.
        if (this._whitespace !== null) {
            // NB: don't populate with only whitespace.
            if (this._output.length !== 0) {
                this._output.push({ node: this._whitespace.node, index: this._whitespace.index });
            }

            this._whitespace = null;
        }

        return this._output;
    }
};

function isPunct(c) {
    return c === '.' || c === '。' || c === '!' || c === '！' || c === '?' || c === '？';
}

function isNumerical(c) {
    return c >= '0' && c <= '9';
}

function isPunctOrNumerical(c) {
    return isPunct(c) || isNumerical(c);
}
