import { StorageChange, Tab } from '../lib/compat.js';
import * as compat from '../lib/compat.js';
import { DomainSettings, loadDomainSetting, toDomainSettings } from '../lib/lib.js';

const B = compat.getBrowser();
const S = compat.getStorage();

function makeIcon(name: string): {[key: string]: string} {
    return {
        '19': `/icons/${name}-19.png`,
        '38': `/icons/${name}-38.png`,
        '48': `/icons/${name}-48.png`,
        '64': `/icons/${name}-64.png`,
        '128': `/icons/${name}-128.png`,
        '256': `/icons/${name}-256.png`,
    };
}

const ICON = makeIcon('jpv');
const ICON_DISABLED = makeIcon('jpv-disabled');

B.onTabUpdated.addListener(async (_tabId, _change, tab) => {
    await updateTab(tab);
});

B.onTabActivated.addListener(async ({tabId}) => {
    let tab = await B.tabsGet(tabId);
    await updateTab(tab);
});

B.onInstalled.addListener(async () => {
    let tabs = await B.tabsQuery({ active: true });

    for (let tab of tabs) {
        await updateTab(tab);
    }
});

S.onStorageChanged.addListener(async (changes: {[key: string]: StorageChange}) => {
    for (let key of Object.keys(changes)) {
        if (!key.startsWith('domain/')) {
            continue;
        }

        let { newValue } = changes[key];
        let setting = toDomainSettings(newValue);

        let [_, host] = key.split('/', 2);
        let tabs = await B.tabsQuery({ active: true });

        for (let tab of tabs) {
            if (!tab.url) {
                continue;
            }

            let url;

            try {
                url = new URL(tab.url);
            } catch (e) {
                continue;
            }

            if (url.host !== host) {
                continue;
            }

            updateIcon(tab, setting);
        }
    }
});

async function updateTab(tab: Tab) {
    if (tab.url === undefined) {
        return;
    }

    let url;

    try {
        url = new URL(tab.url);
    } catch (e) {
        return;
    }

    let setting = await loadDomainSetting(url.host);
    await updateIcon(tab, setting);
}

async function updateIcon(tab: Tab, setting: DomainSettings) {
    if (setting.enabled) {
        await B.setIcon({ tabId: tab.id, path: ICON });
    } else {
        await B.setIcon({ tabId: tab.id, path: ICON_DISABLED });
    }
}
