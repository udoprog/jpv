import { Point, rectContainsAny } from './utils';
import { Boundaries, Bound } from './boundaries';

const DEBUG = false;
const WIDTH = 400;
const HEIGHT = 600;
const PADDING = 10;
const SELECT = true;
const MAX_X_OFFSET = 1024;

let iframe: HTMLIFrameElement | null = null;
let loadListener: (() => void) | null = null;
let lastElement: Element | null = null;
let lastPoint: Point | null = null;
let currentText: string | null = null;
let currentPointOver: number | null = null;

/**
 * Whether we can just press shift to get the popup.
 *
 * We want to avoid triggering this event while the user is typing.
 */
let keyReady: boolean = false;

function isValidStart(el: Element): boolean {
    return el.localName !== "body";
}

function isInlineElement(el: Node | null): boolean {
    if (el instanceof Element) {
        let style = window.getComputedStyle(el as Element);
        return style.display === "inline" || style.display === "inline-block";
    } else {
        return false;
    }
}

/**
 * @returns {Element | null} The bounding element or null if it contains no text.
 */
function getBoundingElement(el: Element): Element | null {
    if (!el.textContent) {
        return null;
    }

    let current = el;

    if (!isValidStart(current)) {
        return null;
    }

    if (isInlineElement(current)) {
        while (isInlineElement(current.parentNode)) {
            current = current.parentNode as Element;
        }

        if (current.parentNode) {
            current = current.parentNode as Element;
        }
    }

    return current;
}

function closeWindow() {
    if (!loadListener || !iframe) {
        return false;
    }

    iframe.removeEventListener('load', loadListener);
    loadListener = null;
    iframe.classList.remove('active');
    iframe.src = '';
    currentText = null;
    return true;
}

interface AdjustResult {
    found: boolean;
    pointOver: number | null;
}

/**
 * Narrows the specified range until it fits a natural word boundary.
 *
 * This is a fairly tricky operation to perform over a DOM, because it contains
 * a bunch of mixed elements, and ranges operate over them.
 *
 * We start by narrowing the range from the right, we have to scan for the last
 * referenced text node, check if it contains a natural paragraph boundary (the
 * various forms of dots, exclamations, ...).
 *
 * Then we repeat the operation from the left.
 *
 * @param {Range} range The range to narrow, until it fits a natural text
 * boundary which is pointed to by the cursor.
 */
function adjustRangeToBoundaries(range: Range, point: Point): AdjustResult {
    let { bounds, pointOver } = walk(range, point);
    let lastCount = 0;

    if (bounds.length === 0) {
        return { found: true, pointOver };
    }

    let current = range.cloneRange();

    if (!rectContainsAny(current.getClientRects(), point)) {
        return { found: false, pointOver };
    }

    let start = 0;
    let end = bounds.length - 1;

    while (start <= end) {
        let { node, index, count } = bounds[start];
        current.setStart(node, index);

        if (!rectContainsAny(current.getClientRects(), point)) {
            break;
        }

        range.setStart(node, index);
        lastCount = count;
        start += 1;
    }

    current.setStart(range.startContainer, range.startOffset);

    while (start <= end) {
        let { node, index } = bounds[end];
        current.setEnd(node, index);

        if (!rectContainsAny(current.getClientRects(), point)) {
            break;
        }

        range.setEnd(node, index);
        end -= 1;
    }

    if (pointOver !== null) {
        return { found: true, pointOver: pointOver - lastCount };
    } else {
        return { found: true, pointOver: null };
    }
}

interface WalkResult {
    bounds: Bound[],
    pointOver: number | null,
}

/**
 * @param {Range} original Original range to scan.
 * @param {Factory}
 * @returns {Range} The walked range range, or null if no valid range was found.
 */
function walk(range: Range, point: Point): WalkResult {
    let node: Node | null = range.startContainer;
    let boundaries = new Boundaries();

    outer:
    while (node) {
        if (node.nodeType === Node.TEXT_NODE) {
            boundaries.populate(node, point);
        } else {
            if (node.firstChild !== null) {
                node = node.firstChild;
                continue;
            }

            if (node.nextSibling !== null) {
                node = node.nextSibling;
                continue;
            }
        }

        if (node === range.endContainer || node.parentNode === range.endContainer) {
            break;
        }

        if (!node.parentNode) {
            break;
        }

        node = node.parentNode.nextSibling;
    }

    return { bounds: boundaries.build(), pointOver: boundaries.getPointOver() };
}

/**
 * Calculate the window position.
 *
 * @param rect The rectangle of the element in where we are placing the popup.
 * @param point The position of the mouse.
 * @returns 
 */
function windowPosition(rect: DOMRect, point: Point) {
    let popupHeight = HEIGHT;
    let popupWidth = WIDTH;
    let padding = PADDING;

    let windowWidth = window.innerWidth;
    let windowHeight = window.innerHeight;

    // Place the window to the right of the element being examined.
    if (rect.x + rect.width + popupWidth + padding * 2 < windowWidth) {
        return {
            x: rect.x + rect.width + padding,
            y: Math.max(Math.min(rect.y, windowHeight - popupHeight - padding), 0),
        };
    }

    // Place the window aligned with the element, but shift to the left if it
    // doesn't fit.
    let x = Math.max(Math.min(rect.x, windowWidth - popupWidth - padding), 0);

    // Test if the window fits below the element.
    if (rect.y + rect.height + popupHeight + padding * 2 < windowHeight) {
        return {
            x,
            y: rect.y + rect.height + padding,
        };
    }

    // Force it to be above the element.
    return {
        x,
        y: rect.y - popupHeight - padding,
    };
}

function openWindow(element: Element | null, point: Point | null) {
    if (!point || !iframe) {
        return;
    }

    if (!element) {
        return;
    }

    element = getBoundingElement(element);

    if (!element) {
        return;
    }

    let textRange = document.createRange();
    textRange.selectNodeContents(element);

    let rect = textRange.getBoundingClientRect();

    let { found, pointOver } = adjustRangeToBoundaries(textRange, point);

    if (!found) {
        return;
    }

    let pos = windowPosition(rect, point);
    let text = textRange.toString().trim();

    if (text === "") {
        return;
    }

    if (SELECT) {
        let s = window.getSelection();

        if (s !== null) {
            if (s.rangeCount > 0) {
                let existing = s.getRangeAt(0);
                existing.setStart(textRange.startContainer, textRange.startOffset);
                existing.setEnd(textRange.endContainer, textRange.endOffset);

                for (let i = 1; i < s.rangeCount; i++) {
                    s.removeRange(s.getRangeAt(i));
                }
            } else {
                s.addRange(textRange);
            }
        }
    }

    if (DEBUG) {
        console.debug(pos);
    }

    if (currentText != text || currentPointOver != pointOver) {
        if (!loadListener) {
            let myIframe = iframe;
            loadListener = () => myIframe.classList.add('active');
            iframe.addEventListener('load', loadListener);
        }

        let search = new URLSearchParams({ embed: "yes", q: text });

        if (pointOver !== null) {
            search.append("analyzeAt", pointOver.toString());
        }

        iframe.src = 'http://localhost:44714?' + search.toString();
        currentText = text;
        currentPointOver = pointOver;
    }

    iframe.style.left = `${pos.x}px`;
    iframe.style.top = `${pos.y}px`;
    iframe.style.width = `${WIDTH}px`;
    iframe.style.height = `${HEIGHT}px`;
    return;
}

function click(e: MouseEvent) {
    lastElement = e.target as Element;
    lastPoint = { x: e.clientX, y: e.clientY };

    if (!e.shiftKey) {
        if (closeWindow()) {
            e.preventDefault();
        }

        return;
    }

    openWindow(lastElement, lastPoint);
    e.preventDefault();
}

function mouseMove(e: MouseEvent) {
    lastElement = e.target as Element;
    lastPoint = { x: e.clientX, y: e.clientY };
    keyReady = true;

    if (e.shiftKey) {
        openWindow(lastElement, lastPoint);
        e.preventDefault();
    }
}

function keyUp(e: KeyboardEvent) {
    keyReady = false;
}

function keyDown(e: KeyboardEvent) {
    if (keyReady && e.key === "Shift") {
        openWindow(lastElement, lastPoint);
        e.preventDefault();
    }
}

if (document.body) {
    let fragment = document.createDocumentFragment();
    iframe = fragment.appendChild(document.createElement('iframe'));

    // set the position to the
    iframe.classList.add('jpv-definitions');
    iframe.src = 'http://localhost:37719';

    document.body.appendChild(iframe);

    document.documentElement.addEventListener('keydown', keyDown);
    document.documentElement.addEventListener('keyup', keyUp);
    document.documentElement.addEventListener('click', click);
    document.documentElement.addEventListener('mousemove', mouseMove);
}
