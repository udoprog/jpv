// domain-specific settings.
const DEFAULT_ENABLED: boolean = true;
const DEFAULT_SELECT: boolean = true;

// settings
const DEFAULT_PORT: number = 44714;
const DEFAULT_DEBUG: boolean = false;
const DEFAULT_WIDTH = 400;
const DEFAULT_HEIGHT = 600;
const DEFAULT_MARGIN = 10;

export interface ControlMessage {
    type: string;
}

export interface DomainSettings {
    enabled: boolean;
    select: boolean;
}

export interface Settings {
    /**
     * The local port to connect to.
     */
    port: number;
    /**
     * Enable debugging.
     */
    debug: boolean;
    /**
     * The width of the popup window.
     */
    width: number;
    /**
     * The height of the popup window.
     */
    height: number;
    /**
     * The requested margin of the popup window.
     */
    margin: number;
}

function toBoolean(data: any, key: string, defaultValue: boolean): void {
    if (data[key] === undefined) {
        data[key] = defaultValue;
    } else {
        data[key] = !!data[key];
    }
}

function toNumber(data: any, key: string, defaultValue: number): void {
    if (data[key] === undefined) {
        data[key] = defaultValue;
    } else if (typeof data[key] === 'string') {
        data[key] = parseInt(data[key]);
    } else if (typeof data[key] === 'number') {
        data[key] = data[key];
    } else {
        data[key] = defaultValue;
    }
}

/**
 * Coerce the maybe structured data into a Settings object.
 */
export function toSettings(data?: any): Settings {
    data = Object.assign({}, data || {});
    toNumber(data, 'port', DEFAULT_PORT);
    toBoolean(data, 'debug', false);
    toNumber(data, 'width', DEFAULT_WIDTH);
    toNumber(data, 'height', DEFAULT_HEIGHT);
    toNumber(data, 'margin', DEFAULT_MARGIN);
    return data as Settings;
}

/**
 * Test if two settings are equal.
 */
export function settingsEqual(a: Settings, b: Settings): boolean {
    return a.port === b.port
        && a.debug === b.debug
        && a.width === b.width
        && a.height === b.height
        && a.margin === b.margin;
}

export function toDomainSettings(data?: any): DomainSettings {
    data = Object.assign({}, data || {});
    toBoolean(data, 'enabled', DEFAULT_ENABLED);
    toBoolean(data, 'select', DEFAULT_SELECT);
    return data as DomainSettings;
}

export async function loadDomainSetting(host: string): Promise<DomainSettings> {
    if (!host) {
        return toDomainSettings();
    }

    let objects = await browser.storage.sync.get(`domain/${host}`);
    return toDomainSettings(objects[`domain/${host}`] || {});
}

export async function loadSettings(): Promise<Settings> {
    let objects = await browser.storage.sync.get('settings');
    return toSettings(objects['settings'] || {});
}

function saveBoolean(output: { [key: string]: any; }, key: string, value: boolean, defaultValue: boolean) {
    if (value !== defaultValue) {
        output[key] = value;
    }
}

function saveNumber(output: { [key: string]: any; }, key: string, value: number, defaultValue: number) {
    if (value !== defaultValue && !isNaN(value)) {
        output[key] = value;
    }
}

export async function saveDomainSettings(domain: string, settings: DomainSettings): Promise<void> {
    let output: {[key: string]: any} = {};

    saveBoolean(output, 'enabled', settings.enabled, DEFAULT_ENABLED);
    saveBoolean(output, 'select', settings.select, DEFAULT_SELECT);

    let update: {[key: string]: any} = {};
    update[`domain/${domain}`] = output;
    await browser.storage.sync.set(update);
}

export async function saveSettings(settings: Settings): Promise<void> {
    let output: {[key: string]: any} = {};

    saveNumber(output, 'port', settings.port, DEFAULT_PORT);
    saveBoolean(output, 'debug', settings.debug, DEFAULT_DEBUG);
    saveNumber(output, 'width', settings.width, DEFAULT_WIDTH);
    saveNumber(output, 'height', settings.height, DEFAULT_HEIGHT);
    saveNumber(output, 'margin', settings.margin, DEFAULT_MARGIN);

    let update: {[key: string]: any} = {};
    update['settings'] = output;
    await browser.storage.sync.set(update);
}

export async function checkAvailable(): Promise<boolean> {
    let global = await loadSettings();
    let request = new Request(`http://127.0.0.1:${global.port}/api/version`, { method: 'HEAD' });

    try {
        let response = await fetch(request);
        return response.status == 200;
    } catch (e) {
        return false;
    }
}
