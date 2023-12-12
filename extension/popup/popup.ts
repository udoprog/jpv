import { Setting, saveSetting, loadSetting } from "../lib/lib";

function setupToggle(power: HTMLInputElement, host: string, setting: Setting) {
    power.addEventListener("click", async () => {
        setting.enabled = !setting.enabled;
        saveSetting(host, setting);
        updateState(power, host, setting);

        let tabs = await browser.tabs.query({ url: `*://${host}/*` });

        for (let tab of tabs) {
            if (!tab.id) {
                continue;
            }

            browser.tabs.sendMessage(tab.id, { type: "update" });
        }
    });

    updateState(power, host, setting);
}

function updateState(power: HTMLInputElement, host: string, setting: Setting) {
    if (setting.enabled) {
        power.classList.add("active");
    } else {
        power.classList.remove("active");
    }
}

async function setup() {
    let tabs = await browser.tabs.query({ active: true, currentWindow: true });

    if (tabs.length !== 1) {
        return;
    }

    let [tab] = tabs;

    if (!tab.url) {
        return;
    }

    let url = new URL(tab.url);

    let power = document.getElementById("power") as HTMLInputElement;
    let domain = document.getElementById("domain") as HTMLDivElement;

    domain.textContent = url.host;
    let setting = await loadSetting(url.host);
    setupToggle(power, url.host, setting);
}

window.addEventListener("load", () => {
    setup();
});
