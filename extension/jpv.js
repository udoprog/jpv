const DEBUG = false;
const WIDTH = 400;
const HEIGHT = 600;
const PADDING = 10;
const SELECT = false;
const FOLLOWMOUSE = false;
const MAX_X_OFFSET = 1024;

let iframe = null;
let loadListener = null;
let currentPhrase = null;

function isValidStart(el) {
    return el.localName !== "body";
}

function isInlineElement(el) {
    let style = window.getComputedStyle(el);
    return style.display === "inline" || style.display === "inline-block";
}

function getPhrase(el) {
    if (!el.textContent) {
        return null;
    }

    let current = el;

    if (!isValidStart(current)) {
        return null;
    }

    if (isInlineElement(current)) {
        while (isInlineElement(current.parentNode)) {
            current = current.parentNode;
        }

        if (current.parentNode) {
            current = current.parentNode;
        }
    }

    if (SELECT) {
        let s = window.getSelection();

        if (s.rangeCount > 0) {
            let first = s.getRangeAt(0);
            first.setStartBefore(current);
            first.setEndAfter(current);
        }
    }

    return { text: current.textContent.trim(), element: current };
}

function closeWindow() {
    if (!loadListener) {
        return false;
    }

    iframe.removeEventListener('load', loadListener);
    loadListener = null;
    iframe.classList.remove('active');
    iframe.src = '';
    currentPhrase = null;
    return true;
}

function windowPosition(element, e) {
    let popupHeight = HEIGHT;
    let popupWidth = WIDTH;
    let padding = PADDING;

    let windowWidth = window.innerWidth;
    let windowHeight = window.innerHeight;

    if (!FOLLOWMOUSE) {
        let range = document.createRange();
        range.selectNodeContents(element);
        range.detach();
        var rect = range.getBoundingClientRect();

        let maxX = e.clientX + MAX_X_OFFSET;

        let pos = {
            x: Math.min(rect.x + rect.width + padding, maxX),
            y: rect.y,
        };

        let neededHeight = pos.y + popupHeight + padding;
        let neededWidth = pos.x + popupWidth + padding;

        if (neededHeight > windowHeight) {
            pos.y -= neededHeight - windowHeight;
        }

        if (neededWidth > windowWidth) {
            pos.x -= neededWidth - windowWidth;
        }

        return pos;
    }

    let pos = { x: e.clientX, y: e.clientY };

    let neededWidth = pos.x + popupWidth + padding * 2;
    let neededHeight = pos.y + popupHeight + padding * 2;

    if (DEBUG) {
        console.debug({ windowWidth, windowHeight });
        console.debug({ neededWidth, neededHeight });
        console.debug(pos);
    }

    if (neededWidth > windowWidth) {
        pos.x -= popupWidth + padding;
    } else {
        pos.x += padding;
    }

    if (neededHeight > windowHeight) {
        pos.y -= (neededHeight - windowHeight) - padding;
    } else {
        pos.y += padding;
    }

    if (pos.y < 0) {
        pos.y = padding;
    }

    return pos;
}

function openWindow(e) {
    let phrase = getPhrase(e.target);

    if (phrase == null) {
        return;
    }

    let { text, element } = phrase;

    let pos = windowPosition(element, e);

    if (DEBUG) {
        console.debug(pos);
    }

    if (currentPhrase != text) {
        if (!loadListener) {
            loadListener = () => iframe.classList.add('active');
            iframe.addEventListener('load', loadListener);
        }

        iframe.src = 'http://localhost:44714?embed=yes&q=' + encodeURIComponent(text);
        currentPhrase = text;
    }

    iframe.style.left = `${pos.x}px`;
    iframe.style.top = `${pos.y}px`;
    iframe.style.width = `${WIDTH}px`;
    iframe.style.height = `${HEIGHT}px`;
    return;
}

function click(e) {
    if (!e.shiftKey) {
        if (closeWindow()) {
            e.preventDefault();
        }

        return;
    }

    openWindow(e);
    e.preventDefault();
}

function mouseMove(e) {
    if (e.shiftKey) {
        openWindow(e);
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

    document.documentElement.addEventListener('click', click);
    document.documentElement.addEventListener('mousemove', mouseMove);
}
