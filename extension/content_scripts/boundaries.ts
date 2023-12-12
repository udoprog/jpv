import {Point, rectContainsAny} from './utils';

export interface Bound {
    node: Node;
    index: number;
    count: number;
}

/**
 * Helper class to calculate boundaries.
 */
export class Boundaries {
    output: Bound[];
    generator: Generator<void, void, string> | null;
    node: Node | null;
    index: number;
    pointOver: number | null;
    count: number;

    constructor() {
        this.output = [];
        this.generator = null;
        this.node = null;
        this.index = 0;
        this.pointOver = null;
        this.count = 0;
    }

    snapshot(offset: number): Bound {
        if (!this.node) {
            throw new Error('no node in snapshot');
        }

        return { node: this.node, index: this.index + offset, count: this.count };
    }

    *buildGenerator(): Generator<void, void, string> {
        function matching(c: string): string | null {
            switch (c) {
                case '「':
                    return '」';
                case '"':
                    return '"';
                case '“':
                    return '”';
                case '(':
                    return ')';
                case '[':
                    return ']';
                case '{':
                    return '}';
                case '<':
                    return '>';
                default:
                    return null;
            }
        }

        let leading = true;
        let expected = null;

        let c = yield;

        while (true) {
            if (isWhiteSpace(c)) {
                while (isWhiteSpace(c)) {
                    c = yield;
                }

                if (leading) {
                    this.output.push(this.snapshot(0));
                }
            }

            leading = false;

            if (expected = matching(c)) {
                let inner = this.snapshot(0);
                while ((c = yield) !== expected) {}
                this.output.push(inner);
                this.output.push(this.snapshot(1));
                c = yield;
                continue;
            }

            if (isPunctuation(c)) {
                while (isPunctuationOrNumerical(c)) {
                    c = yield;
                }

                this.output.push(this.snapshot(0));
                leading = true;
                continue;
            }

            c = yield;
        }
    }

    /**
     * Populate boundaries from a `Node`.
     */
    populate(node: Node, point: Point) {
        if (!this.generator) {
            this.generator = this.buildGenerator();
            this.generator.next();
        }

        let range = document.createRange();
        range.selectNodeContents(node);

        let content = node.textContent;
        this.node = node;

        if (content !== null) {
            for (let i = 0; i < content.length; i++) {
                range.setStart(node, i);
                range.setEnd(node, i + 1);

                if (rectContainsAny(range.getClientRects(), point)) {
                    this.pointOver = this.count;
                }

                this.index = i;
                this.generator.next(content[i]);
                this.count += 1;
            }
        }

        this.node = null;
    }

    build(): Bound[] {
        // free the generator.
        this.generator = null;
        return this.output;
    }

    getPointOver(): number | null {
        return this.pointOver;
    }
};

function isWhiteSpace(c: string): boolean {
    return c === ' ' || c === '　' || c === '\n' || c === '\t';
}

function isPunctuation(c: string): boolean {
    return c === '.' || c === '。' || c === '!' || c === '！' || c === '?' || c === '？';
}

function isNumerical(c: string): boolean {
    return c >= '0' && c <= '9';
}

function isPunctuationOrNumerical(c: string): boolean {
    return isPunctuation(c) || isNumerical(c);
}
