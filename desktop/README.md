# KD Desktop

Windows-x64-App mit Rust, Dioxus und SQLite. Modelle werden separat importiert. Inferenz erfolgt offline mit der mitgelieferten llama.cpp-CPU-Laufzeit b12125. Dioxus verwendet Microsoft WebView2, dessen signierter Offline-Installer mitgeliefert wird.

## Start

1. Das aktuelle `KD-Desktop-<Version>-Setup.exe` aus [GitHub Releases](https://github.com/Akzuwo/KD-K-nstliche-Dummheit/releases) herunterladen und ohne Administratorrechte installieren.
2. KD Desktop über das Startmenü öffnen.
3. **Modell-ZIP importieren**. Empfohlen ist das getrennt ausgelieferte `KD-Modell-0.4.1-<Zeitstempel>.zip` mit Manifest und Modelllizenz. Auch bestehende portable KD-ZIPs mit genau einer GGUF-Datei werden angenommen. Enthaltene Programme werden ignoriert.
4. Nachricht schreiben. Enter sendet, Shift+Enter fügt einen Absatz ein. Der quadratische Stopknopf bricht auch während des Ladens ab. Strg+N beginnt einen neuen Chat.

Die Sidebar bietet Chatsuche (Name und Nachrichteninhalt), Umbenennen und Löschen über das Dreipunktmenü. Das obere KD-Menü wechselt das aktive Modell. **Modelle** verwaltet alle installierten Versionen. Ein Wechsel gilt für die nächste Antwort; bestehende Antworten zeigen ihre Modellherkunft. Während einer Generierung sind Wechsel und Modelllöschung gesperrt. **Einstellungen** bietet Hell/Dunkel und CPU/CUDA.

Markdown unterstützt Überschriften, Listen, Tabellen, Zitate, Inline-Code und Codeblöcke mit Kopierknopf. Codeblöcke sind lesbar formatiert, ohne Syntax-Highlighting. Externe Bilder und HTML werden nicht ausgeführt. Ein Linkklick kopiert die Adresse; zum Öffnen in einen Browser einfügen. Die Schrift stammt lokal aus Windows. Keine externen Fonts oder Markenassets.

## Datenverwaltung

`%LOCALAPPDATA%\KD Desktop\` enthält:

| Pfad | Inhalt |
|---|---|
| `chats.sqlite3`, `-wal`, `-shm` | Chats, Nachrichten und Einstellungen |
| `models/<UUID>/` | Importierte GGUF und internes Manifest |
| `logs/inference-latest.log` | Diagnose des letzten lokalen Modellstarts |
| `webview/` | Lokales WebView-Profil |
| `app.lock` | Schutz gegen gleichzeitigen Zugriff zweier App-Fenster |
| `updates/` | Geprüfte GitHub-Installer und Protokoll der letzten automatischen Installation |

Die App komplett schliessen, bevor dieser Ordner gesichert oder kopiert wird. Interaktive Deinstallation fragt zusätzlich, ob alle App-Daten dauerhaft entfernt werden sollen; **Nein ist voreingestellt**. Stille Deinstallation behält sie immer. Deinstallation und Modelllöschung berühren ursprüngliche ZIPs, Projektmodelle und alte Builds nicht. Modell- und Chatlöschung benötigen jeweils eine Bestätigung in der App.

Für Entwicklung/Tests überschreibt `KD_DATA_DIR` den Datenordner; `KD_RUNTIME_DIR` zeigt auf einen Ordner mit `cpu/llama-server.exe` und optional `cuda/llama-server.exe`. Ein gestarteter Prozess erhält diese Variablen beim Start. Die App bindet ihren Modellserver nur an einen zufälligen authentifizierten IPv4-Loopback-Port. Ein Windows-Job-Object beendet den Modellprozess auch bei hartem App-Ende.

## Modellpaketformat

```text
KD-Modell-0.4.1.zip
  kd-model.json
  model.gguf
```

```json
{
  "format": 1,
  "name": "KD",
  "version": "0.4.1",
  "file": "model.gguf",
  "sha256": "64 hexadezimale Zeichen der GGUF-Prüfsumme",
  "context": 4096
}
```

Kompatibel sind eigenständige GGUF-v2/v3-Textmodelle mit Chat-Template, die die gepinnte llama.cpp-Version unterstützt. Kontext laut Manifest 1024–32768, maximal 32 GiB pro GGUF und 4096 Archiveinträge. Keine HF-Safetensors, losen LoRA-Adapter, verteilten GGUFs oder Multimodal-Projektoren. Mehrere Dateien erfordern ein Manifest, das die konkrete GGUF nennt. Identische Gewichte werden anhand SHA-256 als Duplikat erkannt; unterschiedliche Versionen können gleichzeitig installiert bleiben.

Der Import prüft Pfade, doppelte Windows-Pfadnamen, Symlinks, Grössen, CRC beim Lesen, GGUF-Header und die Manifest-Prüfsumme. Legacy-Pakete ohne Manifest erhalten beim Import eine berechnete Prüfsumme, aber keinen Herkunftsnachweis. Vollständige Tensor-/Architekturkompatibilität wird beim tatsächlichen Laden geprüft; ein gültiger Header garantiert sie nicht. Fehlgeschlagener Import entfernt nur seinen eigenen temporären Ordner. Fremde EXEs, DLLs und Skripte werden niemals aus Modell-ZIPs übernommen.

Ein neues Paket erzeugen:

```powershell
desktop/target/release/kd-tool.exe package --model models/KD-SFT-v041-qwen3-long-q8.gguf --output builds/KD-Modell-NEUER-ZEITSTEMPEL.zip --name KD --version 0.4.1
```

## Architektur

- `src/ui.rs`: Dioxus-Komponenten, Signale und asynchrone Ereignisse. Dateidialog/ZIP-Import und Inferenz laufen ausserhalb des UI-Threads.
- `src/store.rs`: SQLite mit Fremdschlüsseln, WAL, Parametern und persistenter Kennzeichnung unterbrochener Antworten. Nur vollständige Gesprächspaare kommen in den nächsten Kontext.
- `src/models.rs`: Paketformat, SHA-256, Import-Staging, atomare Installation, Modellkatalog und eingegrenzte Löschung.
- `src/engine.rs`: Rust-Steuerung des mitgelieferten llama.cpp-Prozesses, Windows-Job, health check, Chat-Template-/Tokenzählung, Kürzen ganzer Gesprächspaare, SSE und Abbruch.
- `src/markdown.rs`: Markdown und HTML-Sanitizer; keine Remote-Bilder.
- `assets/`: eingebettetes CSS, eigene SVG-Icons im Rust-Code, eigenes KD-App-Icon, kleine WebView-Helfer für Kopieren, Tastatur, Fokus und Scrollen. Keine JavaScript-Frameworks.
- `src/bin/kd-tool.rs`: getrenntes Entwicklerwerkzeug für Pakete und echte Inferenztests.

Die Generierungsparameter und der neutrale deutsche Systemtext stammen aus `app/LocalEngine.cs` und v0.4.1: Temperatur 0,5, Wiederholungsstrafe 1,05, DRY 0,2, höchstens 900 neue Tokens, nativer Non-Thinking-Modus und Seed 50117. Es gibt keinen Absurditäts-Systemprompt. Die Engine startet pro Antwort neu und gibt RAM danach frei; das verursacht eine kurze Ladephase. Lange Eingaben werden mit dem echten Modell-Template gezählt. Alte vollständige Paare werden bei Platzmangel ausgelassen, die letzte Eingabe wird nie still abgeschnitten. Abgebrochene/fehlerhafte Antworten sind sichtbar und aus dem Modellkontext ausgeschlossen.

## Entwickeln und bauen

Windows x64 mit Rust MSVC, Visual Studio C++ Build Tools/Windows SDK, WebView2. Cargo.lock ist Teil der Quellen. Werkzeuge und Pakete dürfen beim Entwickeln heruntergeladen werden, die fertige App inferiert lokal.

```powershell
cargo test --locked --manifest-path desktop/Cargo.toml --lib
cargo check --locked --manifest-path desktop/Cargo.toml
cargo fmt --manifest-path desktop/Cargo.toml --check
.\desktop\scripts\build.ps1
```

Das Buildskript bezieht die gepinnte llama.cpp-CPU-Laufzeit und Inno Setup 7.1.1 mit SHA-256-Prüfung sowie den Microsoft-signierten WebView2-Installer. Die x64-CRT stammt aus den installierten Visual Studio C++ Build Tools. Jeder Build erhält einen neuen Zeitstempel. Die Ausgabe enthält Setup, App-Staging, getrenntes Paketwerkzeug, Abhängigkeitslizenzen und SHA-256-Dateimanifest.

Echte CPU-Prüfung mit isolierten Daten (Ziel darf noch nicht existieren):

```powershell
$build = (Get-Content -LiteralPath desktop/last-build.txt -Raw).Trim()
& (Join-Path $build 'kd-tool.exe') live-test --data validation-desktop/NEUER-ZEITSTEMPEL --zip PFAD_ZUM_MODELL_ZIP --runtime (Join-Path $build 'app/runtime')
```

## Updates und Releases

Ab v1.1.0 fragt die installierte App beim Start und danach alle sechs Stunden die jüngste stabile Version aus GitHub Releases ab. Ein neuer Installer wird im Hintergrund geladen und mit Dateigrösse und GitHub-SHA-256 geprüft. Das Update startet automatisch, sobald das App-Fenster regulär geschlossen wird. Nach der Installation steht die neue Version beim nächsten Start bereit. Chats, Modelle, Einstellungen und der Installationspfad werden weiterverwendet. In den Einstellungen steht der aktuelle Updatestatus. Bei einem Installationsfehler liegt das Protokoll in `updates/install.log`. Ältere App-Versionen werden einmal über den aktuellen Installer aktualisiert.

Der Workflow [release-desktop.yml](../.github/workflows/release-desktop.yml) baut und prüft Windows-Installer bei Änderungen auf `main` und bei manuellen Läufen. Ein gepushter Tag `vX.Y.Z`, dessen Version mit [Cargo.toml](Cargo.toml) übereinstimmt, veröffentlicht das Setup und dessen SHA-256-Datei auf GitHub. Der Workflow setzt das Release als aktuell. Die Modellinferenz bleibt offline. [GitHub Release API](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)

## Optionales NVIDIA CUDA

`scripts/prepare-cuda.ps1 -Destination NEUER_ORDNER` lädt ausschliesslich die offiziellen b12125-CUDA-12-Archive und verifiziert ihre GitHub-SHA-256-Digests. Der Entwickler kopiert den geprüften Unterordner `cuda` nach `<Installationsordner>/runtime/cuda`. Die App lädt **keine** Runtime selbst aus dem Netz und nimmt keine DLLs aus Modellpaketen an. Danach ist die CUDA-Auswahl in den Einstellungen verfügbar. CPU bleibt als manuelle Rückfalloption erhalten. CUDA ist auf diesem Intel-Laptop nicht getestet und wird im CPU-Installer nicht mitgeliefert.

## Prüfgrenzen

Die Gestaltung orientiert sich eng an der ChatGPT-Weboberfläche mit eigenem KD-Branding. Eine Pixelidentität zu sämtlichen aktuellen ChatGPT-Varianten ist nicht belegt. Es gibt keine Voice-Funktion, Websuche, Bildgenerierung oder Reasoning-Auswahl. Antworten des v0.4.1-Modells behalten ihre dokumentierten Sprach-/Qualitätsprobleme. Der Installer ist nicht digital signiert. Ein sauberer Zweitrechner-/VM-Test und echter NVIDIA-CUDA-Test stehen aus.

Technische Primärquellen: [Dioxus Desktop](https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/), [llama.cpp](https://github.com/ggml-org/llama.cpp), [Inno Setup](https://jrsoftware.org/inno-setup).

## KD Desktop v1.0: Navigation und Darstellung

App-Version und Modellversionen sind unabhängig. KD Desktop v1.0 kann beispielsweise das unveränderte KD-Modell v0.4.1 nutzen. App-Name und Modellwahl sind im Kopfbereich getrennt. AppId und Datenverzeichnis bleiben beim Update erhalten.

Der Hell-/Dunkel-Schalter ist direkt oben rechts und in den Einstellungen erreichbar. Beide zeigen denselben Zustand, die Auswahl wird lokal gespeichert. Beim ersten Start wird die Betriebssystempräferenz übernommen; eine gespeicherte Auswahl hat Vorrang.

Ein geöffneter Navdrawer besitzt kein `inert`-Attribut. Nur der geschlossene Drawer wird gesperrt. Bei Dioxus 0.7 sperrt auch `inert="false"` sämtliche Kindkontrollen, weshalb ein Boolean hier nicht verwendet werden darf.

Der Regressionstest `desktop/scripts/test-navigation.ps1` startet dieselben Dioxus-Komponenten in einem isolierten nativen WebView2. Er prüft den tatsächlichen DOM-Zustand, Fokus und Zeigertreffer sowie die Rust-Handler für Sidebar, Chats, Suche, Umbenennen und Einstellungen. Die Theme-Prüfung vergleicht gerenderte Farben und den SQLite-Wert. Es werden keine Desktop-Eingaben gesendet und keine Benutzerdaten verwendet. Die erforderliche Cargo-Funktion `ui-check` gehört nicht zum normalen Installer-Build.
