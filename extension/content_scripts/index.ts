import { Point, rectContainsAny } from './utils';
import { Boundaries, Bound } from './boundaries';
import { Setting, loadSetting, toSetting } from '../lib/lib';

const DEBUG = false;
const PADDING = 10;
const WIDTH = 400;
const HEIGHT = 600;

interface OldCursor {
    element: HTMLElement;
    cursor: string;
}

interface ContentMessage {
    type: 'loaded';
}

class Globals {
    window: Window;
    target: HTMLElement;
    errorWindow: HTMLDivElement | null;
    iframe: HTMLIFrameElement | null;
    visible: boolean;
    lastElement: HTMLElement | null;
    lastPoint: Point | null;
    currentText: string | null;
    currentPointOver: number | null;
    setting: Setting;
    oldCursor: OldCursor | null;
    started: boolean;
    error: boolean;
    windowPos: Point;
    messageHandle: (e: MessageEvent) => void;
    mouseMoveHandle: (e: MouseEvent) => void;
    clickHandle: (e: MouseEvent) => void;
    keyUpHandle: (e: KeyboardEvent) => void;

    constructor(window: Window, target: HTMLElement) {
        this.window = window;
        this.target = target;
        this.errorWindow = null;
        this.iframe = null;
        this.visible = false;
        this.lastElement = null;
        this.lastPoint = null;
        this.currentText = null;
        this.currentPointOver = null;
        this.setting = toSetting(null);
        this.oldCursor = null;
        this.started = false;
        this.error = true;
        this.windowPos = { x: 0, y: 0 };
        this.messageHandle = this.message.bind(this);
        this.mouseMoveHandle = this.mouseMove.bind(this);
        this.clickHandle = this.click.bind(this);
        this.keyUpHandle = this.keyUp.bind(this);
    }

    /**
     * Reset globals to their default values.
     */
    reset() {
        this.clearCursor();
        this.errorWindow = null;
        this.iframe = null;
        this.visible = false;
        this.lastElement = null;
        this.lastPoint = null;
        this.currentText = null;
        this.currentPointOver = null;
        this.setting = toSetting(null);
        this.oldCursor = null;
        this.error = true;
        this.windowPos = { x: 0, y: 0 };
    }

    /**
     * Set a cursor for the given element.
     *
     * @param element The element to modify cursor for.
     */
    setCursor(element: HTMLElement) {
        if (this.oldCursor !== null) {
            if (this.oldCursor.element === element) {
                return;
            }

            // Restore old cursor since element has changed.
            this.oldCursor.element.style.cursor = this.oldCursor.cursor;
            this.oldCursor = null;
        }

        if (this.oldCursor === null) {
            this.oldCursor = {
                element,
                cursor: element.style.cursor,
            };

            element.style.cursor = 'pointer';
        }
    }

    /**
     * Clear the cursor that is over the current element.
     */
    clearCursor() {
        if (this.oldCursor !== null) {
            this.oldCursor.element.style.cursor = this.oldCursor.cursor;
            this.oldCursor = null;
        }
    }

    setWindowState() {
        if (this.iframe === null || this.errorWindow === null) {
            return;
        }

        let a: HTMLElement = this.iframe;
        let o: HTMLElement = this.errorWindow;

        if (this.error) {
            a = this.errorWindow;
            o = this.iframe;
        }

        if (this.visible) {
            a.classList.add('active');
            o.classList.remove('active');
        } else {
            a.classList.remove('active');
            o.classList.remove('active');
        }

        if (this.visible) {
            a.style.left = `${this.windowPos.x}px`;
            a.style.top = `${this.windowPos.y}px`;
            a.style.width = `${WIDTH}px`;
            a.style.height = `${HEIGHT}px`;
        } else {
            a.style.left = "0px";
            a.style.right = "0px";
            a.style.width = "0px";
            a.style.height = "0px";
        }

        o.style.left = "0px";
        o.style.right = "0px";
        o.style.width = "0px";
        o.style.height = "0px";
    }

    /**
     * Open hover window.
     */
    openWindow() {
        if (this.lastPoint === null || this.iframe === null || this.errorWindow === null || this.lastElement === null) {
            return;
        }

        let element = getBoundingElement(this.lastElement);

        if (!element) {
            return;
        }

        let textRange = document.createRange();
        textRange.selectNodeContents(element);

        let rect = textRange.getBoundingClientRect();

        let { found, pointOver } = adjustRangeToBoundaries(textRange, this.lastPoint);

        if (!found) {
            return;
        }

        let pos = windowPosition(rect, this.lastPoint);
        let text = textRange.toString().trim();

        if (text === '') {
            return;
        }

        this.setCursor(element);

        if (this.setting.select) {
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

        if (!this.visible) {
            this.visible = true;
        }

        if (this.currentText != text || this.currentPointOver != pointOver) {
            let message = { type: "update", text } as UpdateMessage;

            if (pointOver !== null) {
                message.analyze_at_char = pointOver;
            }

            if (!!this.iframe.contentWindow) {
                this.iframe.contentWindow.postMessage(message, '*');
            }

            this.currentText = text;
            this.currentPointOver = pointOver;
        }

        this.windowPos = pos;
        this.setWindowState();
        return;
    }

    /**
     * Close the current window.
     *
     * @returns {boolean} True if the window was closed, false otherwise.
     */
    closeWindow(): boolean {
        if (!this.visible || this.iframe === null || this.errorWindow === null) {
            return false;
        }

        this.visible = false;
        this.setWindowState();
        this.currentText = null;
        this.clearCursor();
        return true;
    }

    click(e: MouseEvent) {
        this.lastElement = e.target as HTMLElement;
        this.lastPoint = { x: e.clientX, y: e.clientY };

        if (!e.shiftKey) {
            if (this.closeWindow()) {
                e.preventDefault();
            }

            return;
        }

        if (this.visible) {
            this.openWindow();
            e.preventDefault();
        }
    }

    message(e: MessageEvent) {
        if (this.iframe === null || this.errorWindow === null || this.iframe.contentWindow === null) {
            return;
        }

        if (e.source !== this.iframe.contentWindow) {
            return;
        }

        let data = e.data as ContentMessage;

        if (data.type === 'loaded') {
            this.error = false;
            this.setWindowState();
        }
    }

    mouseMove(e: MouseEvent) {
        this.lastElement = e.target as HTMLElement;
        this.lastPoint = { x: e.clientX, y: e.clientY };

        if (e.shiftKey && e.buttons === 0) {
            this.openWindow();
            e.preventDefault();
        } else {
            this.clearCursor();
        }
    }

    keyUp(e: KeyboardEvent) {
        if (e.key === 'Shift') {
            this.clearCursor();
        }
    }

    async setUp(setting: Setting) {
        this.setting = setting;

        if (this.iframe !== null) {
            return;
        }

        this.errorWindow = document.createElement('div');

        this.window.addEventListener('message', this.messageHandle);

        // set the position to the
        this.errorWindow.classList.add('jpv-window');
        this.errorWindow.innerHTML = '\
            <div id="jpv-error">\
                <div class="jpv-title">The jpv service is not available</div>\
                <div class="jpv-content">Make sure it\'s running on your computer.</div>\
            </div>';

        this.iframe = document.createElement('iframe');
        this.iframe.classList.add('jpv-window');
        this.iframe.src = 'http://localhost:44714?embed=yes';

        this.target.appendChild(this.errorWindow);
        this.target.appendChild(this.iframe);

        document.documentElement.addEventListener('click', this.clickHandle);
        document.documentElement.addEventListener('mousemove', this.mouseMoveHandle);
        document.documentElement.addEventListener('keyup', this.keyUpHandle);
    }

    async tearDown() {
        if (this.iframe === null || this.errorWindow === null) {
            return;
        }

        this.target.removeChild(this.errorWindow);
        this.target.removeChild(this.iframe);

        window.removeEventListener('message', this.messageHandle);
        document.documentElement.removeEventListener('click', this.clickHandle);
        document.documentElement.removeEventListener('mousemove', this.mouseMoveHandle);
        document.documentElement.removeEventListener('keyup', this.keyUpHandle);
        this.reset();
    }

    async initialize(setting: Setting) {
        if (setting.enabled) {
            await this.setUp(setting);
        } else {
            await this.tearDown();
        }
    }

    async start() {
        if (this.started) {
            return;
        }

        this.started = true;
        let setting = await loadSetting(location.host);
        await this.initialize(setting);
    }
}

/**
 * Global variables.
 */
const G = new Globals(window, document.body);

// Start the content script.
G.start();

browser.storage.sync.onChanged.addListener((changes) => {
    let { newValue } = changes[`domain/${location.host}`];

    if (newValue !== undefined) {
        G.initialize(toSetting(newValue));
    }
});

interface UpdateMessage {
    type: "update",
    text: string;
    analyze_at_char?: number;
}

function isValidStart(el: Element): boolean {
    return el.localName !== 'body';
}

function isInlineElement(el: Node | null): boolean {
    if (el instanceof Element) {
        let style = window.getComputedStyle(el as Element);
        return style.display === 'inline' || style.display === 'inline-block';
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
