# The web UI's own words. Dictionary content (glosses) is not translated.

not-found = There is nothing here
loading = Loading…
back = Back
save = Save
cancel = Cancel

# The language picker.
language = Language
language-option = { $native } ({ $english })

# The gloss language picker.
gloss-language = Gloss language
gloss-language-hint = Entries without glosses in this language show their English ones.
gloss-language-eng = English
gloss-language-dut = Dutch
gloss-language-fre = French
gloss-language-ger = German
gloss-language-hun = Hungarian
gloss-language-rus = Russian
gloss-language-slv = Slovenian
gloss-language-spa = Spanish
gloss-language-swe = Swedish

# Editing a dictionary.
index-id = Id
index-id-hint = The unique identifier of the dictionary, must only contain [a-z], [A-Z], and [0-9].
index-id-invalid = Must be a non-empty sequence of [a-z], [A-Z], and [0-9].
index-format = Format
index-url = URL
index-source = Source
index-source-url = Download
index-source-file = Local file
index-path = File
index-path-hint = A gzipped dictionary file on the machine the service runs on.
index-path-empty = Choose a file
index-path-browse = Browse…
index-path-choose = Choose a dictionary file
index-url-empty = Must be non-empty
index-description = Description
index-help = Help Page
index-delete = Delete
index-update = Update
index-no-update = Do not update

# Kanji details.
kanji-radicals = Radicals
kanji-strokes = Strokes

# The analysis of the query.
analyze-cycle = { $index } / { $count } (click character to cycle)
analyze-hint = Hint:
analyze-hint-text = Click character for substring search

# The config page.
config-dictionaries = Dictionaries
config-ocr = OCR
config-log = Log
config-help = About
config-help-tip = Go to the help page for this dictionary
config-help-missing = No help page specified
config-not-installed = not installed
config-updated-tip = Has been updated and will be applied on Save
config-edit-tip = Change { $index }
config-index-actions = Actions for { $index }
config-ocr-not-installed = OCR support is not installed
config-ocr-support = OCR Support
config-new-dictionary = New dictionary
config-install-all = Install all
config-install-all-tip = Install all missing dictionaries

# The theme switch.
theme = Theme
theme-system = System
theme-light = Light
theme-dark = Dark

# Errors the UI words itself. Errors from the service are shown as written.
error-not-connected = Not connected to the service
error-connection-lost = Connection to the service was lost before it answered

# The window's top bar and the credits.
title = Japanese Dictionary
by-author = by { $author }
search = Search
configure = Configure
open-window = Open in big window
window = Window
credits-made = Made with ❤️ by { $author }, freely available forever under the { $mit } or { $apache }
credits-apache = Apache 2.0 license
credits-data = This application uses { $jmdict }, { $kanjidic }, { $enamdict }, and { $kradfile } which is the property of the { $edrdg } and is used in conformance with its { $licence }.
credits-licence = licence

# The search page.
nothing-to-analyze = Nothing to analyze
type-something = Type something in the prompt
translation = Translation:
phrases = Phrases
names = Names
kanji = Kanji
results = Results
resize-results = Resize the results and kanji
tab-count = { $title } ({ $count })
tab-kanji-details = Kanji details: { $kanji }
tab-settings = Settings
showing-phrases = Showing { $shown } out of { $total } phrases
showing-characters = Showing { $shown } out of { $total } characters
show-more = Show more
mode-default = default
mode-default-tip = Do not process input at all
mode-hiragana-tip = Process input as Hiragana
mode-katakana-tip = Treat input as Katakana
capture-clipboard = Capture clipboard
mode = Input mode
missing-dictionaries = Dictionaries missing:
fix-in-settings = Fix in Settings
ocr-missing = OCR support is enabled but not installed
disable-ocr = Disable

# An entry.
entry-other-kana = Other kana
entry-other-kanji = Other kanji
entry-applies-to = Applies to
entry-english = English
entry-inflection = Result based on inflection:
entry-reset = Reset
entry-actions = Entry actions
entry-copy = Copy word
entry-copy-reading = Copy reading
entry-open-data = Open entry data
entry-open-data-detail = Entry #{ $sequence }
# The inflection's name, such as "Past" (from jpv's inflection list).
entry-tutorial = Tutorial for `{ $form }`
entry-tutorial-tip = Tutorial for { $form }

# Background tasks.
task-cancel = Cancel { $task }
task-cancelling = Cancelling ...
task-cancelled = { $task } was cancelled

# Choosing a file on the service's machine.
file-browser-location = Location
file-browser-up = Parent directory
file-browser-hidden = Show hidden files
file-browser-home = Home
file-browser-documents = Documents
file-browser-downloads = Downloads
file-browser-empty = Nothing here
file-browser-open = Open
file-browser-name = File name
file-browser-replaces = A file with this name exists and will be replaced.
