import { Settings } from "../lib/lib";
import * as lib from "../lib/lib";

let defaultSettings = lib.toSettings();

interface Elements {
    debug: HTMLInputElement;
    port: HTMLInputElement;
    width: HTMLInputElement;
    height: HTMLInputElement;
    margin: HTMLInputElement;
    reset: HTMLButtonElement;
}

function setupToggle(elements: Elements, settings: Settings) {
    elements.debug.addEventListener('change', e => {
        settings.debug = elements.debug.checked;
        lib.saveSettings(settings);
        updateFromSettings(elements, settings);
    });

    elements.port.addEventListener('change', e => {
        settings.port = parseInt(elements.port.value);
        lib.saveSettings(settings);
        updateFromSettings(elements, settings);
    });

    elements.width.addEventListener('change', e => {
        settings.width = parseInt(elements.width.value);
        lib.saveSettings(settings);
        updateFromSettings(elements, settings);
    });

    elements.height.addEventListener('change', e => {
        settings.height = parseInt(elements.height.value);
        lib.saveSettings(settings);
        updateFromSettings(elements, settings);
    });

    elements.margin.addEventListener('change', e => {
        settings.margin = parseInt(elements.margin.value);
        lib.saveSettings(settings);
        updateFromSettings(elements, settings);
    });

    elements.reset.addEventListener('click', e => {
        let settings = lib.toSettings();
        lib.saveSettings(settings);
        updateFromSettings(elements, settings);
    });
}

function updateFromSettings(elements: Elements, settings: Settings) {
    elements.debug.disabled = false;
    elements.debug.checked = settings.debug;

    elements.port.disabled = false;
    elements.port.value = settings.port.toString();

    elements.width.disabled = false;
    elements.width.value = settings.width.toString();

    elements.height.disabled = false;
    elements.height.value = settings.height.toString();

    elements.margin.disabled = false;
    elements.margin.value = settings.margin.toString();

    elements.reset.disabled = lib.settingsEqual(settings, defaultSettings);
}

async function setup() {
    let elements = {
        debug: document.getElementById('debug') as HTMLInputElement,
        port: document.getElementById('port') as HTMLInputElement,
        width: document.getElementById('width') as HTMLInputElement,
        height: document.getElementById('height') as HTMLInputElement,
        margin: document.getElementById('margin') as HTMLInputElement,
        reset: document.getElementById('reset') as HTMLButtonElement,
    } as Elements;

    let settings = await lib.loadSettings();
    updateFromSettings(elements, settings);
    setupToggle(elements, settings);
}

setup();
