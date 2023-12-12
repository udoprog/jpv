export interface Point {
    x: number;
    y: number;
}

/**
 * Test if the collection of rectangles contains the given point.
 */
export function rectContainsAny(rects: DOMRectList, point: Point) {
    for (let i = 0; i < rects.length; i++) {
        if (rectContains(rects[i], point)) {
            return true;
        }
    }

    return false;
}

/**
 * Test if a dom rectangle contains the given point.
 */
export function rectContains(rect: DOMRect, point: Point) {
    return rect.left <= point.x && rect.right >= point.x && rect.top <= point.y && rect.bottom >= point.y;
}
