import { Setting, saveSetting, loadSetting } from "../lib/lib";

interface Elements {
    power: HTMLInputElement;
    domain: HTMLDivElement;
    status: HTMLDivElement;
    hint: HTMLDivElement;
    select: HTMLInputElement;
}

function setupToggle(elements: Elements, host: string, setting: Setting) {
    elements.select.addEventListener("change", e => {
        setting.select = elements.select.checked;
        saveSetting(host, setting);
    });

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

    elements.select.checked = setting.select;
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

    if (tab.url === undefined) {
        return;
    }

    let url = new URL(tab.url);

    if (!url.host) {
        return;
    }

    let elements = {
        power: document.getElementById("power") as HTMLInputElement,
        domain: document.getElementById("domain") as HTMLDivElement,
        status: document.getElementById("status") as HTMLDivElement,
        hint: document.getElementById("hint") as HTMLDivElement,
        select: document.getElementById("select") as HTMLInputElement,
    } as Elements;

    elements.power.classList.add("clickable");
    elements.domain.textContent = url.host;
    elements.select.disabled = false;
    let setting = await loadSetting(url.host);
    setupToggle(elements, url.host, setting);
}

window.addEventListener("load", () => {
    setup();
});
