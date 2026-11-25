import Adw from 'gi://Adw';
import Gio from 'gi://Gio';

import {ExtensionPreferences, gettext as _} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

export default class JapaneseDictionaryPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const settings = this.getSettings();

        const page = new Adw.PreferencesPage({
            title: _('General'),
            icon_name: 'preferences-system-symbolic',
        });

        const group = new Adw.PreferencesGroup({
            title: _('Clipboard'),
        });

        const captureClipboard = new Adw.SwitchRow({
            title: _('Capture clipboard'),
            subtitle: _('Send everything copied to the clipboard to the dictionary'),
        });

        settings.bind('capture-clipboard-enabled', captureClipboard, 'active', Gio.SettingsBindFlags.DEFAULT);

        group.add(captureClipboard);
        page.add(group);
        window.add(page);

        // Keep the settings object alive for as long as the window is.
        window._settings = settings;
    }
}
