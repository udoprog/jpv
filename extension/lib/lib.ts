export interface ControlMessage {
    type: string;
}

export interface Setting {
    host: string,
    enabled: boolean;
}

export async function loadSetting(host: string): Promise<Setting> {
    let objects = await browser.storage.sync.get(`by-site/${host}`);
    let data = objects[`by-site/${host}`] || {};

    data.host = host;

    if (data.enabled === undefined) {
        data.enabled = true;
    }

    return data as Setting;
}

export async function saveSetting(setting: Setting): Promise<void> {
    let update: {[key: string]: any} = {};
    update[`by-site/${setting.host}`] = { enabled: setting.enabled };
    await browser.storage.sync.set(update);
    console.log("saved", update);
}
