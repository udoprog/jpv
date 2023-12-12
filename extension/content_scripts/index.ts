import { Point, rectContainsAny } from './utils';
import { Boundaries, Bound } from './boundaries';
import { Setting, loadSetting, toSetting } from '../lib/lib';

const DEBUG = false;
const WIDTH = 400;
const HEIGHT = 600;
const PADDING = 10;

// Global state.
let gIframe: HTMLIFrameElement | null = null;
let gVisible: boolean = false;
let gLastElement: HTMLElement | null = null;
let gLastPoint: Point | null = null;
let gCurrentText: string | null = null;
let gCurrentPointOver: number | null = null;
let gSetting: Setting = toSetting(null);
let gOldCursor: OldCursor | null = null;

interface UpdateMessage {
    text: string;
    analyze_at_char?: number;
}

interface OldCursor {
    element: HTMLElement;
    cursor: string;
}

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
function getBoundingElement(el: HTMLElement): HTMLElement | null {
    if (!el.textContent) {
        return null;
    }

    let current = el;

    if (!isValidStart(current)) {
        return null;
    }

    if (isInlineElement(current)) {
        while (isInlineElement(current.parentNode)) {
            current = current.parentNode as HTMLElement;
        }

        if (current.parentNode) {
            current = current.parentNode as HTMLElement;
        }
    }

    return current;
}

function setCursor(element: HTMLElement) {
    if (gOldCursor !== null) {
        if (gOldCursor.element === element) {
            return;
        }

        // Restore old cursor since element has changed.
        gOldCursor.element.style.cursor = gOldCursor.cursor;
        gOldCursor = null;
    }

    if (gOldCursor === null) {
        gOldCursor = {
            element,
            cursor: element.style.cursor,
        };

        element.style.cursor = 'pointer';
    }
}

function clearCursor() {
    if (gOldCursor !== null) {
        gOldCursor.element.style.cursor = gOldCursor.cursor;
        gOldCursor = null;
    }
}

function closeWindow() {
    if (!gVisible || !gIframe) {
        return false;
    }

    gVisible = false;
    gIframe.classList.remove('active');
    gCurrentText = null;
    clearCursor();
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

function openWindow(element: HTMLElement | null, point: Point | null) {
    if (!point || !gIframe) {
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

    setCursor(element);

    if (gSetting.select) {
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

    if (!gVisible) {
        gIframe.classList.add('active');
        gVisible = true;
    }

    if (gCurrentText != text || gCurrentPointOver != pointOver) {
        let message = { text } as UpdateMessage;

        if (pointOver !== null) {
            message.analyze_at_char = pointOver;
        }

        if (!!gIframe.contentWindow) {
            gIframe.contentWindow.postMessage(message, '*');
        }

        gCurrentText = text;
        gCurrentPointOver = pointOver;
    }

    gIframe.style.left = `${pos.x}px`;
    gIframe.style.top = `${pos.y}px`;
    gIframe.style.width = `${WIDTH}px`;
    gIframe.style.height = `${HEIGHT}px`;
    return;
}

function click(e: MouseEvent) {
    gLastElement = e.target as HTMLElement;
    gLastPoint = { x: e.clientX, y: e.clientY };

    if (!e.shiftKey) {
        if (closeWindow()) {
            e.preventDefault();
        }

        return;
    }

    if (gVisible) {
        openWindow(gLastElement, gLastPoint);
        e.preventDefault();
    }
}

function mouseMove(e: MouseEvent) {
    gLastElement = e.target as HTMLElement;
    gLastPoint = { x: e.clientX, y: e.clientY };

    if (e.shiftKey && e.buttons === 0) {
        openWindow(gLastElement, gLastPoint);
        e.preventDefault();
    } else {
        clearCursor();
    }
}

function keyUp(e: KeyboardEvent) {
    if (e.key === 'Shift') {
        clearCursor();
    }
}

async function setUp(setting: Setting) {
    gSetting = setting;

    if (!document.body || gIframe !== null) {
        return;
    }

    let fragment = document.createDocumentFragment();
    gIframe = fragment.appendChild(document.createElement('iframe'));

    // set the position to the
    gIframe.classList.add('jpv-definitions');
    gIframe.src = 'http://localhost:44714?embed=yes';

    document.body.appendChild(gIframe);

    document.documentElement.addEventListener('click', click);
    document.documentElement.addEventListener('mousemove', mouseMove);
    document.documentElement.addEventListener('keyup', keyUp);
}

async function tearDown() {
    if (!document.body || gIframe === null) {
        return;
    }

    document.body.removeChild(gIframe);

    document.documentElement.removeEventListener('click', click);
    document.documentElement.removeEventListener('mousemove', mouseMove);
    document.documentElement.removeEventListener('keyup', keyUp);

    gIframe = null;
    gVisible = false;
    gLastElement = null;
    gLastPoint = null;
    gCurrentText = null;
    gCurrentPointOver = null;
    clearCursor();
    gSetting = toSetting(null);
}

async function initialize(setting: Setting) {
    if (setting.enabled) {
        await setUp(setting);
    } else {
        await tearDown();
    }
}

async function start() {
    let setting = await loadSetting(location.host);
    await initialize(setting);
}

browser.storage.sync.onChanged.addListener((changes) => {
    let { newValue } = changes[`by-site/${location.host}`];

    if (newValue !== undefined) {
        initialize(toSetting(newValue));
    }
});

start();
