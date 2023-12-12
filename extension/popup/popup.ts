import { Setting, saveSetting, loadSetting } from "../lib/lib";

function setupToggle(toggle: HTMLInputElement, setting: Setting) {
    toggle.addEventListener("click", async () => {
        setting.enabled = !setting.enabled;
        saveSetting(setting);
        updateState(toggle, setting);

        let tabs = await browser.tabs.query({ url: `*://${setting.host}/*` });

        for (let tab of tabs) {
            if (!tab.id) {
                continue;
            }

            browser.tabs.sendMessage(tab.id, { type: "update" });
        }
    });

    toggle.classList.add("active");
    updateState(toggle, setting);
}

function updateState(toggle: HTMLInputElement, setting: Setting) {
    if (setting.enabled) {
        toggle.textContent = `Disable ${setting.host}`;
    } else {
        toggle.textContent = `Enable ${setting.host}`;
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

    let toggle = document.getElementById("toggle") as HTMLInputElement;

    if (!toggle) {
        return;
    }

    let setting = await loadSetting(url.host);
    setupToggle(toggle, setting);
}

setup();
