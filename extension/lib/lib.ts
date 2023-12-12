export interface ControlMessage {
    type: string;
}

export interface Setting {
    enabled: boolean;
}

export function toSetting(data: any): Setting {
    data = Object.assign({}, data || {});

    if (data.enabled === undefined) {
        data.enabled = true;
    } else {
        data.enabled = !!data.enabled;
    }

    return data as Setting;
}

export async function loadSetting(host: string): Promise<Setting> {
    let objects = await browser.storage.sync.get(`by-site/${host}`);
    let data = objects[`by-site/${host}`] || {};
    return toSetting(data);
}

export async function saveSetting(host: string, setting: Setting): Promise<void> {
    let value: {[key: string]: any} = {};

    if (!setting.enabled) {
        value.enabled = false;
    }

    let update: {[key: string]: any} = {};
    update[`by-site/${host}`] = value;

    await browser.storage.sync.set(update);
}
