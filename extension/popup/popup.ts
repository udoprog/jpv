import { Setting, saveSetting, loadSetting } from "../lib/lib";

interface Elements {
    power: HTMLInputElement;
    domain: HTMLDivElement;
    status: HTMLDivElement;
    hint: HTMLDivElement;
}

function setupToggle(elements: Elements, host: string, setting: Setting) {
    elements.power.addEventListener("click", async () => {
        setting.enabled = !setting.enabled;
        saveSetting(host, setting);
        updateState(elements, host, setting);

        let tabs = await browser.tabs.query({ url: `*://${host}/*` });

        for (let tab of tabs) {
            if (!tab.id) {
                continue;
            }

            browser.tabs.sendMessage(tab.id, { type: "update" });
        }
    });

    updateState(elements, host, setting);
}

function updateState(elements: Elements, host: string, setting: Setting) {
    if (setting.enabled) {
        elements.power.classList.add("active");
        elements.hint.classList.add("active");
        elements.status.textContent = "enabled";
    } else {
        elements.power.classList.remove("active");
        elements.hint.classList.remove("active");
        elements.status.textContent = "disabled";
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

    let elements = {
        power: document.getElementById("power") as HTMLInputElement,
        domain: document.getElementById("domain") as HTMLDivElement,
        status: document.getElementById("status") as HTMLDivElement,
        hint: document.getElementById("hint") as HTMLDivElement,
    } as Elements;

    elements.domain.textContent = url.host;
    let setting = await loadSetting(url.host);
    setupToggle(elements, url.host, setting);
}

window.addEventListener("load", () => {
    setup();
});
