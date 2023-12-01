interface Bound {
    node: Node;
    index: number;
}

interface Quote extends Bound {
    expected: string;
}


/**
 * Helper class to calculate boundaries.
 */
export default class Boundaries {
    quote: Quote | null;
    count: number;
    output: Bound[];
    whiteSpace: Bound | null;
    leading: boolean;

    constructor() {
        this.quote = null;
        this.count = 0;
        this.output = [];
        this.whiteSpace = null;
        this.leading = true;
    }

    /**
     * Populate boundaries from a node.
     *
     * @param {string} content Content to scan.
     * @returns {number[]} Boundaries found.
     */
    populate(node: Node) {
        let content = node.textContent;

        if (content === null) {
            return;
        }

        for (let i = 0; i < content.length; i++) {
            this.count += 1;
            let c = content[i];

            if (isWhiteSpace(c)) {
                if (this.whiteSpace === null) {
                    this.whiteSpace = { node, index: i };
                }

                continue;
            }

            if (this.quote !== null) {
                if (this.quote.expected !== c) {
                    continue;
                }

                this.output.push({ node: this.quote.node, index: this.quote.index });
                this.output.push({ node, index: i + 1 });
                this.quote = null;
            }

            if (c === '「') {
                this.quote = { node, index: i, expected: '」' };
                continue;
            }

            if (c === '\"') {
                this.quote = { node, index: i, expected: '\"' };
                continue;
            }

            if (this.leading && this.whiteSpace !== null) {
                this.output.push({ node, index: i });
            }

            this.leading = false;
            this.whiteSpace = null;

            if (isPunct(c)) {
                let u = i;

                while (u < content.length && isPunctOrNumerical(content[u + 1])) {
                    u += 1;
                }

                if (content.length == u || isWhiteSpace(content[u + 1])) {
                    continue;
                }

                i = u;
                this.output.push({ node, index: i + 1 });
            }
        }
    }

    build(): Bound[] {
        // Populate trailing whitespace.
        if (this.whiteSpace !== null) {
            // NB: don't populate with only whitespace.
            if (this.output.length !== 0) {
                this.output.push({ node: this.whiteSpace.node, index: this.whiteSpace.index });
            }

            this.whiteSpace = null;
        }

        return this.output;
    }
};

function isWhiteSpace(c: string): boolean {
    return c === ' ' || c === '　' || c === '\n' || c === '\t';
}

function isPunct(c: string): boolean {
    return c === '.' || c === '。' || c === '!' || c === '！' || c === '?' || c === '？';
}

function isNumerical(c: string): boolean {
    return c >= '0' && c <= '9';
}

function isPunctOrNumerical(c: string): boolean {
    return isPunct(c) || isNumerical(c);
}
