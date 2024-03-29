
interface OnUpdatedChangeInfo {
}

/* storage types */
export interface StorageChange {
    /** The old value of the item, if there was an old value. */
    oldValue?: any;
    /** The new value of the item, if there is a new value. */
    newValue?: any;
}

export interface Tab {
    url?: string;
    id?: number;
}

interface EventListener<T> {
    addListener(cb: T): void;
}

export interface Storage {
    /**
     * Get configuration.
     *
     * @param key Configuration key to get.
     */
    storageGet(key: string): Promise<{ [key: string]: any; }>;

    /**
     * Set configuration.
     *
     * @param data Configuration updates to perform.
     */
    storageSet(data: { [key: string]: any; }): Promise<void>;

    /**
     * Register a callback for when extension settings have been changed.
     */
    onStorageChanged: EventListener<(changes: { [key: string]: StorageChange }) => void>;
}

export interface Browser {
    /**
     * Update icon for the tab.
     */
    setIcon(details: { tabId?: number, path: { [key: string]: string } }): Promise<void>;

    /**
     * Get a tab.
     */
    tabsGet(tabId: number): Promise<Tab>;

    /**
     * Query for tabs.
     */
    tabsQuery(query: { active: boolean, currentWindow?: boolean }): Promise<Tab[]>;

    /**
     * Register callback for when tab has been updated.
     */
    onTabUpdated: EventListener<(tabId: number, changeInfo: OnUpdatedChangeInfo, tab: Tab) => void>;

    /**
     * Register callback for when tab has been activated.
     */
    onTabActivated: EventListener<(activeInfo: { tabId: number; windowId: number; }) => void>;

    /**
     * Register a callback for when the extension has been installed.
     */
    onInstalled: EventListener<() => void>;
}

export function getBrowser(): Browser {
    if (typeof browser !== 'undefined') {
        return {
            setIcon: browser.browserAction.setIcon,
            tabsGet: browser.tabs.get,
            tabsQuery: browser.tabs.query,
            onTabUpdated: browser.tabs.onUpdated,
            onTabActivated: browser.tabs.onActivated,
            onInstalled: browser.runtime.onInstalled,
        };
    }

    if (typeof chrome !== 'undefined') {
        return {
            setIcon: (key) => {
                return chrome.action.setIcon(key);
            },
            tabsGet: (tabId) => {
                return new Promise((resolve) => chrome.tabs.get(tabId, resolve));
            },
            tabsQuery: (query) => {
                return new Promise((resolve) => chrome.tabs.query(query, resolve));
            },
            onTabUpdated: chrome.tabs.onUpdated,
            onTabActivated: chrome.tabs.onActivated,
            onInstalled: chrome.runtime.onInstalled,
        };
    }

    throw new Error("Unsupported browser");
}

export function getStorage(): Storage {
    if (typeof browser !== 'undefined') {
        return {
            storageGet: browser.storage.sync.get.bind(browser.storage.sync),
            storageSet: browser.storage.sync.set.bind(browser.storage.sync),
            onStorageChanged: browser.storage.sync.onChanged,
        };
    }

    if (typeof chrome !== 'undefined') {
        return {
            storageGet: (key) => {
                return new Promise((resolve) => chrome.storage.sync.get(key, resolve));
            },
            storageSet: (data) => {
                return new Promise((resolve) => chrome.storage.sync.set(data, resolve));
            },
            onStorageChanged: chrome.storage.onChanged,
        };
    }

    throw new Error("Unsupported browser");
}
