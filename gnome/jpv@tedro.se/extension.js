import GObject from 'gi://GObject';
import St from 'gi://St';
import Shell from 'gi://Shell';
import Meta from 'gi://Meta';
import Gio from 'gi://Gio';

import {Extension, gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';

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

const Indicator = GObject.registerClass(
class Indicator extends PanelMenu.Button {
    #refreshInProgress = false;

    _init(extension) {
        super._init(0.0, _('My Shiny Indicator'));

        this.extension = extension;

        this.add_child(new St.Icon({
            icon_name: 'se.tedro.JapaneseDictionary',
            style_class: 'system-status-icon',
        }));

        let openUi = new PopupMenu.PopupMenuItem(_('Open UI'));

        openUi.connect('activate', () => {
            extension.proxy.GetPortRemote((port, error) => {
                if (error) {
                    console.error(error);
                } else {
                    let p = port[0];
                    let url = 'http://localhost:' + p;
                    console.log(url);
                    console.log(Gio.app_info_launch_default_for_uri(url, null));
                }
            });
        });

        this.menu.addMenuItem(openUi);

        const metaDisplay = Shell.Global.get().get_display();
        const selection = metaDisplay.get_selection();
        this._setupSelectionTracking(selection);
    }

    _setupSelectionTracking (selection) {
        this.selection = selection;
        this._selectionOwnerChangedId = selection.connect('owner-changed', (selection, selectionType, selectionSource) => {
            this._onSelectionChange(selection, selectionType, selectionSource).catch(console.error);
        });
    }

    async _onSelectionChange (selection, selectionType, selectionSource) {
        if (selectionType === Meta.SelectionType.SELECTION_CLIPBOARD) {
            this._refreshIndicator();
        }
    }

    async _refreshIndicator () {
        if (this.#refreshInProgress) {
            return;
        }

        this.#refreshInProgress = true;

        try {
            const result = await this.#getClipboardContent();

            if (result) {
                console.log(result);

                this.extension.proxy.SendClipboardDataRemote(result.mimeType, result.data, (response, error, list) => {
                    if (error) {
                        console.error(error);
                    } else {
                        console.log(response, error);
                    }
                });
            }
        }
        catch (e) {
            console.error('Failed to refresh indicator');
            console.error(e);
        }
        finally {
            this.#refreshInProgress = false;
        }
    }

    async #getClipboardContent () {
        const mimetypes = [
            'text/plain;charset=utf-8',
            'text/plain',
            'image/gif',
            'image/png',
            'image/jpg',
            'image/jpeg',
            'image/webp',
            'image/svg+xml',
            'text/html',
        ];

        for (let type of mimetypes) {
            let result = await new Promise(resolve => this.extension.clipboard.get_content(CLIPBOARD_TYPE, type, (clipBoard, bytes) => {
                if (bytes === null || bytes.get_size() === 0) {
                    resolve(null);
                    return;
                }

                resolve(new ClipboardEntry(type, bytes.get_data()));
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
