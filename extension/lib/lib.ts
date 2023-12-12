export interface ControlMessage {
    type: string;
}

export interface Setting {
    enabled: boolean;
    select: boolean;
}

function toBoolean(data: any, key: string, defaultValue: boolean): void {
    if (data[key] === undefined) {
        data[key] = defaultValue;
    } else {
        data[key] = !!data[key];
    }
}

export function toSetting(data: any): Setting {
    data = Object.assign({}, data || {});
    toBoolean(data, "enabled", true);
    toBoolean(data, "select", true);
    return data as Setting;
}

export async function loadSetting(host: string): Promise<Setting> {
    let objects = await browser.storage.sync.get(`by-site/${host}`);
    let data = objects[`by-site/${host}`] || {};
    return toSetting(data);
}

function saveBoolean(output: { [key: string]: any; }, key: string, value: boolean) {
    if (!value) {
        output[key] = false;
    }
}

export async function saveSetting(host: string, setting: Setting): Promise<void> {
    let output: {[key: string]: any} = {};

    saveBoolean(output, "enabled", setting.enabled);
    saveBoolean(output, "select", setting.select);

    let update: {[key: string]: any} = {};
    update[`by-site/${host}`] = output;
    await browser.storage.sync.set(update);
}
