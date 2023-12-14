export interface ControlMessage {
    type: string;
}

export interface Setting {
    enabled: boolean;
    select: boolean;
}

export interface GlobalSetting {
    port: number;
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
    } else {
        data[key] = !!data[key];
    }
}

export function toSetting(data: any): Setting {
    data = Object.assign({}, data || {});
    toBoolean(data, 'enabled', true);
    toBoolean(data, 'select', true);
    return data as Setting;
}

export function toGlobalSetting(data: any): GlobalSetting {
    data = Object.assign({}, data || {});
    toNumber(data, 'port', 44714);
    return data as GlobalSetting;
}

export async function loadSetting(host: string): Promise<Setting> {
    if (!host) {
        return toSetting(null);
    }

    let objects = await browser.storage.sync.get(`domain/${host}`);
    return toSetting(objects[`domain/${host}`] || {});
}

export async function loadGlobalSetting(): Promise<GlobalSetting> {
    let objects = await browser.storage.sync.get('global');
    return toGlobalSetting(objects['global'] || {});
}

function saveBoolean(output: { [key: string]: any; }, key: string, value: boolean) {
    if (!value) {
        output[key] = false;
    }
}

export async function saveSetting(host: string, setting: Setting): Promise<void> {
    let output: {[key: string]: any} = {};

    saveBoolean(output, 'enabled', setting.enabled);
    saveBoolean(output, 'select', setting.select);

    let update: {[key: string]: any} = {};
    update[`domain/${host}`] = output;
    await browser.storage.sync.set(update);
}

export async function checkAvailable(): Promise<boolean> {
    let global = await loadGlobalSetting();
    let request = new Request(`http://127.0.0.1:${global.port}/api/version`, { method: 'HEAD' });

    try {
        let response = await fetch(request);
        return response.status == 200;
    } catch (e) {
        return false;
    }
}
