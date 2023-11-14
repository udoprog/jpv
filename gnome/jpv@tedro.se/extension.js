import GObject from 'gi://GObject';
import St from 'gi://St';
import Shell from 'gi://Shell';
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
];

const JapaneseDictionaryInterface = `
<node>
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
        _init(title) {
            let settings = new Gio.Settings({
                schema_id: 'se.tedro.japanese-dictionary.plugins',
            });

            super._init(title, settings.get_boolean('capture-clipboard-enabled'));

            this.connect('toggled', (item, state) => {
                settings.set_boolean('capture-clipboard-enabled', state);
            });
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

        let openUi = new PopupMenu.PopupMenuItem(_('Open UI'));

        openUi.connect('activate', () => {
            extension.proxy.GetPortRemote((port, error) => {
                if (error) {
                    console.error(error);
                } else {
                    let p = port[0];
                    Gio.app_info_launch_default_for_uri(`http://localhost:${p}`, null);
                }
            });
        });

        this._toggleClipboard = new ClipboardToggle(_('Capture clipboard'));

        this.menu.addMenuItem(openUi);
        this.menu.addMenuItem(this._toggleClipboard);

        const metaDisplay = Shell.Global.get().get_display();
        const selection = metaDisplay.get_selection();
        this._selection = selection;

        if (this._toggleClipboard.state) {
            this._setup();
            this.add_style_pseudo_class('capture');
        }

        this._toggleClipboard.connect('toggled', (item, state) => {
            if (state) {
                this._setup();
                this.add_style_pseudo_class('capture');
            } else {
                this._destroy();
                this.remove_style_pseudo_class('capture');
            }
        });
    }

    _setup() {
        this._currentSelection = this._selection.connect('owner-changed', (selection, selectionType, selectionSource) => {
            if (selectionType === Meta.SelectionType.SELECTION_CLIPBOARD) {
                this._sendClipboardData().catch(console.error);
            }
        });
    }

    _destroy() {
        if (this._currentSelection) {
            this._selection.disconnect(this._currentSelection);
            this._currentSelection = null;
        }
    }

    async _sendClipboardData () {
        if (this.#sendInProgress) {
            return;
        }

        this.#sendInProgress = true;

        try {
            const result = await this.#getClipboardContent();

            if (result) {
                this.extension.proxy.SendClipboardDataRemote(result.mimeType, result.data, (_response, error) => {
                    if (error) {
                        console.error(error);
                    }
                });
            }
        } catch (e) {
            console.error('Failed to send clipboard data');
            console.error(e);
        } finally {
            this.#sendInProgress = false;
        }
    }

    async #getClipboardContent () {
        for (let atom of ATOMS) {
            let result = await new Promise(resolve => this.extension.clipboard.get_content(CLIPBOARD_TYPE, atom, (_cb, bytes) => {
                if (bytes === null || bytes.get_size() === 0) {
                    resolve(null);
                    return;
                }

                resolve(new ClipboardEntry(atom, bytes.get_data()));
            }));

            if (result) return result;
        }

        return null;
    }
});

export default class IndicatorExampleExtension extends Extension {
    enable() {
        this.clipboard = St.Clipboard.get_default(),
        this.proxy = new JapaneseDictionaryProxy(Gio.DBus.session, 'se.tedro.JapaneseDictionary', '/se/tedro/JapaneseDictionary');
        this._indicator = new Indicator(this);
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        this._indicator.destroy();
        this._indicator = null;
    }
}
