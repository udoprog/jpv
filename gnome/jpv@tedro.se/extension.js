import GObject from 'gi://GObject';
import St from 'gi://St';
import Meta from 'gi://Meta';
import Gio from 'gi://Gio';

import {Extension, gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';

// List of atoms that we care about.
const ATOMS = [
    "UTF8_STRING",
    "STRING",
    'text/plain;charset=utf-8',
    'text/plain',
    'image/png',
    'image/tiff',
    'image/webp',
    'image/jpeg',
    'image/jpg',
];

const JapaneseDictionaryInterface = `
<!DOCTYPE node PUBLIC
    "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
    "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd" >
<node xmlns:doc="http://www.freedesktop.org/dbus/1.0/doc.dtd">
    <interface name="se.tedro.JapaneseDictionary">
        <method name="GetPort">
            <arg type="q" direction="out" name="port" />
        </method>
        <method name="SendClipboardData">
            <arg type="s" direction="in" name="mimetype" />
            <arg type="ay" direction="in" name="data" />
        </method>
    </interface>
</node>
`;
const JapaneseDictionaryProxy = Gio.DBusProxy.makeProxyWrapper(JapaneseDictionaryInterface);

const CLIPBOARD_TYPE = St.ClipboardType.CLIPBOARD;

class ClipboardEntry {
    constructor (mimeType, data) {
        this.mimeType = mimeType;
        this.data = data;
    }
}

const ClipboardToggle = GObject.registerClass(
class ClipboardToggle extends PopupMenu.PopupSwitchMenuItem {
    _init(title, settings) {
        super._init(title, settings.get_boolean('capture-clipboard-enabled'));

        this.connect('toggled', (_item, state) => {
            if (settings.get_boolean('capture-clipboard-enabled') !== state) {
                settings.set_boolean('capture-clipboard-enabled', state);
            }
        });

        // Disconnected automatically when this item is destroyed.
        settings.connectObject('changed::capture-clipboard-enabled', (_settings, key) => {
            const state = settings.get_boolean(key);

            if (this.state !== state) {
                this.setToggleState(state);
                this.emit('toggled', state);
            }
        }, this);
    }
});

const Indicator = GObject.registerClass(
class Indicator extends PanelMenu.Button {
    #sendInProgress = false;

    _init(extension) {
        super._init(0.0, _('Japanese Dictionary by John-John Tedro'));

        this.extension = extension;

        this.add_child(new St.Icon({
            icon_name: 'se.tedro.JapaneseDictionary',
            style_class: 'system-status-icon',
        }));

        this.add_style_class_name('japanese-dictionary-icon');

        const openDictionary = new PopupMenu.PopupMenuItem(_('Open dictionary'));

        openDictionary.connect('activate', () => {
            extension.proxy.GetPortRemote((port, error) => {
                if (error) {
                    Main.notifyError(_('Failed to open dictionary'), `${error}`);
                } else {
                    const url = `http://localhost:${port[0]}`;
                    Gio.AppInfo.launch_default_for_uri(url, null);
                }
            });
        });

        this._toggleClipboard = new ClipboardToggle(_('Capture clipboard 📋'), extension.settings);

        this.menu.addMenuItem(openDictionary);
        this.menu.addMenuItem(this._toggleClipboard);

        this._selection = null;
        this._ownerChangedId = 0;

        this._toggleClipboard.connect('toggled', (_item, state) => this._setCapture(state));
        this._setCapture(this._toggleClipboard.state);

        this.connect('destroy', () => this._unwatchSelection());
    }

    _setCapture(enabled) {
        if (enabled) {
            this._watchSelection();
            this.add_style_pseudo_class('capture');
        } else {
            this._unwatchSelection();
            this.remove_style_pseudo_class('capture');
        }
    }

    _watchSelection() {
        if (this._selection) {
            return;
        }

        this._selection = global.display.get_selection();

        this._ownerChangedId = this._selection.connect('owner-changed', (_selection, type, _source) => {
            if (type === Meta.SelectionType.SELECTION_CLIPBOARD) {
                this._sendClipboardData().catch(e => console.error(e));
            }
        });
    }

    _unwatchSelection() {
        if (this._selection && this._ownerChangedId) {
            this._selection.disconnect(this._ownerChangedId);
        }

        this._ownerChangedId = 0;
        this._selection = null;
    }

    async _sendClipboardData() {
        if (this.#sendInProgress) {
            return;
        }

        this.#sendInProgress = true;

        try {
            const result = await this.#getClipboardContent();

            if (result) {
                await new Promise((resolve, reject) => this.extension.proxy.SendClipboardDataRemote(result.mimeType, result.data, (_response, error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }));
            }
        } catch (e) {
            console.error('Failed to send clipboard data');
            console.error(e);
        } finally {
            this.#sendInProgress = false;
        }
    }

    async #getClipboardContent() {
        for (let atom of ATOMS) {
            let result = await new Promise(resolve => this.extension.clipboard.get_content(CLIPBOARD_TYPE, atom, (_cb, bytes) => {
                if (bytes === null || bytes.get_size() === 0) {
                    resolve();
                    return;
                }

                resolve(new ClipboardEntry(atom, bytes.get_data()));
            }));

            if (result) {
                return result;
            }
        }

        return null;
    }
});

export default class JapaneseDictionaryExtension extends Extension {
    enable() {
        this.settings = this.getSettings();
        this.clipboard = St.Clipboard.get_default();
        this.proxy = new JapaneseDictionaryProxy(Gio.DBus.session, 'se.tedro.JapaneseDictionary', '/se/tedro/JapaneseDictionary');
        this._indicator = new Indicator(this);
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        this._indicator?.destroy();
        this._indicator = null;
        this.proxy = null;
        this.clipboard = null;
        this.settings = null;
    }
}
