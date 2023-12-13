import { Setting, loadSetting, toSetting } from "../lib/lib";

browser.tabs.onUpdated.addListener(async (tabId) => {
    let tab = await browser.tabs.get(tabId);
    await updateTab(tab);
});

browser.tabs.onActivated.addListener(async ({tabId}) => {
    let tab = await browser.tabs.get(tabId);
    await updateTab(tab);
});

browser.runtime.onInstalled.addListener(async () => {
    let tabs = await browser.tabs.query({ active: true });

    for (let tab of tabs) {
        await updateTab(tab);
    }
});

browser.storage.sync.onChanged.addListener(async (changes) => {
    for (let key of Object.keys(changes)) {
        if (!key.startsWith("by-site/")) {
            continue;
        }

        let { newValue } = changes[key];
        let setting = toSetting(newValue);

        let [_, host] = key.split("/", 2);
        let tabs = await browser.tabs.query({ active: true });

        for (let tab of tabs) {
            if (!tab.url) {
                continue;
            }

            let url = new URL(tab.url);

            if (url.host !== host) {
                continue;
            }

            updateIcon(tab, setting);
        }
    }
});

async function updateTab(tab: browser.tabs.Tab) {
    if (tab.url === undefined) {
        return;
    }

    let url = new URL(tab.url);
    let setting = await loadSetting(url.host);
    updateIcon(tab, setting);
}

function updateIcon(tab: browser.tabs.Tab, setting: Setting) {
    if (setting.enabled) {
        browser.browserAction.setIcon({ tabId: tab.id, path: "/icons/jpv-256.png" });
    } else {
        browser.browserAction.setIcon({ tabId: tab.id, path: "/icons/jpv-disabled-256.png" });
    }
}
