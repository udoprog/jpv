import { Setting, saveSetting, loadSetting, checkAvailable } from '../lib/lib.js';

interface Elements {
    power: HTMLInputElement;
    domain: HTMLDivElement;
    status: HTMLDivElement;
    hint: HTMLDivElement;
    select: HTMLInputElement;
    unavailable: HTMLDivElement;
}

function setupToggle(elements: Elements, available: boolean, host: string, setting: Setting) {
    if (available) {
        elements.select.addEventListener('change', e => {
            setting.select = elements.select.checked;
            saveSetting(host, setting);
        });
    }

    if (available) {
        elements.power.addEventListener('click', async () => {
            setting.enabled = !setting.enabled;
            saveSetting(host, setting);
            updateState(elements, available, setting);
        });
    }

    elements.select.checked = setting.select;
    updateState(elements, available, setting);
}

function updateState(elements: Elements, available: boolean, setting: Setting) {
    if (setting.enabled && available) {
        elements.power.classList.add('active');
        elements.hint.classList.add('active');
        elements.status.textContent = 'enabled';
        elements.select.disabled = false;
    } else {
        elements.power.classList.remove('active');
        elements.hint.classList.remove('active');
        elements.status.textContent = 'disabled';
        elements.select.disabled = true;
    }

    if (available) {
        elements.unavailable.classList.remove('active');
    } else {
        elements.unavailable.classList.add('active');
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
        power: document.getElementById('power') as HTMLInputElement,
        domain: document.getElementById('domain') as HTMLDivElement,
        status: document.getElementById('status') as HTMLDivElement,
        hint: document.getElementById('hint') as HTMLDivElement,
        select: document.getElementById('select') as HTMLInputElement,
        unavailable: document.getElementById('unavailable') as HTMLDivElement,
    } as Elements;

    let available = await checkAvailable();
    elements.power.classList.add('clickable');
    elements.domain.textContent = url.host;
    let setting = await loadSetting(url.host);
    setupToggle(elements, available, url.host, setting);
}

window.addEventListener('load', () => {
    setup();
});
