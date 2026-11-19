---
name: "KD Desktop"
description: "Lokale KD-Chats im freigegebenen ChatGPT-artigen visuellen System."
colors:
  light-bg: "#fff"
  light-side: "#f9f9f9"
  light-surface: "#f4f4f4"
  light-hover: "#ececec"
  light-text: "#0d0d0d"
  light-muted: "#626262"
  light-line: "#e5e5e5"
  light-bubble: "#f4f4f4"
  light-composer: "#f4f4f4"
  light-button: "#0d0d0d"
  light-on-button: "#fff"
  light-danger: "#b42318"
  dark-bg: "#212121"
  dark-side: "#171717"
  dark-surface: "#2f2f2f"
  dark-hover: "#343434"
  dark-text: "#ececec"
  dark-muted: "#ababab"
  dark-line: "#3d3d3d"
  dark-bubble: "#303030"
  dark-composer: "#303030"
  dark-button: "#fff"
  dark-on-button: "#111"
  dark-danger: "#ff9990"
  focus: "#4b91d1"
  local-status: "#538961"
  avatar-bg: "#d2ddd3"
  avatar-text: "#25452a"
typography:
  display:
    fontFamily: "'Segoe UI', Arial, sans-serif"
    fontSize: "32px"
    fontWeight: 500
    lineHeight: 1.25
    letterSpacing: "-.8px"
  headline:
    fontFamily: "'Segoe UI', Arial, sans-serif"
    fontSize: "30px"
    fontWeight: 500
    letterSpacing: "-.6px"
  title:
    fontFamily: "'Segoe UI', Arial, sans-serif"
    fontSize: "20px"
    fontWeight: 600
  body:
    fontFamily: "'Segoe UI', Arial, sans-serif"
    fontSize: "14px"
  message:
    fontFamily: "'Segoe UI', Arial, sans-serif"
    fontSize: "16px"
    lineHeight: 1.75
  label:
    fontFamily: "'Segoe UI', Arial, sans-serif"
    fontSize: "12px"
    fontWeight: 500
  code:
    fontFamily: "Consolas, 'Cascadia Code', monospace"
    fontSize: "13px"
    lineHeight: 1.6
rounded:
  inline: "4px"
  control: "8px"
  field: "9px"
  code: "14px"
  menu: "15px"
  dialog: "20px"
  button: "22px"
  bubble: "24px"
  composer: "26px"
  circle: "50%"
spacing:
  tight: "6px"
  control: "8px"
  compact: "10px"
  field: "12px"
  section: "16px"
  text: "18px"
  message: "20px"
  dialog: "24px"
  gutter: "28px"
components:
  button-primary:
    backgroundColor: "{colors.light-button}"
    textColor: "{colors.light-on-button}"
    rounded: "{rounded.button}"
    padding: "11px 18px"
  button-primary-dark:
    backgroundColor: "{colors.dark-button}"
    textColor: "{colors.dark-on-button}"
    rounded: "{rounded.button}"
    padding: "11px 18px"
  button-secondary:
    backgroundColor: "{colors.light-bg}"
    textColor: "{colors.light-text}"
    rounded: "{rounded.button}"
    padding: "11px 18px"
  button-danger:
    backgroundColor: "{colors.light-danger}"
    textColor: "{colors.light-bg}"
    rounded: "{rounded.button}"
    padding: "11px 18px"
  button-icon:
    rounded: "{rounded.control}"
    width: "34px"
    height: "34px"
  button-send:
    backgroundColor: "{colors.light-button}"
    textColor: "{colors.light-on-button}"
    rounded: "{rounded.circle}"
    width: "36px"
    height: "36px"
  navigation-row:
    rounded: "{rounded.control}"
    padding: "10px"
    width: "100%"
  text-input:
    backgroundColor: "{colors.light-surface}"
    rounded: "{rounded.field}"
    padding: "12px"
    width: "100%"
  composer:
    backgroundColor: "{colors.light-composer}"
    rounded: "{rounded.composer}"
    padding: "15px 14px 10px"
  user-bubble:
    backgroundColor: "{colors.light-bubble}"
    rounded: "{rounded.bubble}"
    padding: "10px 20px"
  dialog:
    backgroundColor: "{colors.light-bg}"
    rounded: "{rounded.dialog}"
    width: "640px"
---

# Design System: KD Desktop

## Overview

**Creative North Star: "ChatGPT-like chat, KD identity"**

Die verbindliche visuelle Referenz ist die ChatGPT-artige Chatoberfläche mit eigener KD-Identität. Das System verwendet zurückhaltende Grauflächen, eine schmale Chatnavigation, grosszügig gesetzten Gesprächsinhalt und einen weich gerundeten Composer. Die vom Nutzer festgelegte Referenz bestimmt auch die lokale Segoe-Schrift; eine alternative Marken- oder Schriftwelt ist nicht vorgesehen.

KD erscheint als eigene Wortmarke und eigenes App-Icon. Die Desktop-App ist ein lokaler Dioxus-WebView; das visuelle System wird aus assets/app.css und src/ui.rs dokumentiert. Die Dokumentation bestätigt die implementierten Werte, keine Pixelgleichheit mit einer aktuellen ChatGPT-Variante. Vollständige visuelle Klick-QA wurde vom Nutzer zurückgestellt; Screenshots und bestandene visuelle Tests werden hier nicht behauptet.

**Key Characteristics:**
- Neutrale helle und dunkle Flächen mit klarer Text-Hierarchie.
- Schmale Navigation und zentrierter Gesprächsinhalt.
- Runde Eingabe- und Aktionsformen; sparsame Tiefenwirkung.
- Eigene KD-Wortmarke, eigene SVG-Icons und lokales App-Icon.

Geltungsbereich: ausschliesslich die eigenständige Rust-/Dioxus-Anwendung unter `desktop/`. Quellen: [Produktvorgaben](../PRODUCT.md), [Stylesheet](assets/app.css), [Komponenten](src/ui.rs), [App-Dokumentation](README.md). Historische Terminal-, WPF- und Weboberflächen gehören nicht zu diesem System. Die Frontmatter benennt explizite Licht-/Dunkelwerte; zur Laufzeit schaltet das vorhandene `data-theme` zwischen denselben semantischen CSS-Variablen um. Frontmatter und Sidecar sind Dokumentation, keine zusätzliche Laufzeitkonfiguration.

## Colors

Die Palette bleibt überwiegend neutral. Flächen unterscheiden sich durch kleine Helligkeitsabstände; Text und Hauptaktionen erhalten den stärksten Kontrast.

### Primary

- **Monochrome Hauptaktion:** `light-button` / `dark-button` mit den jeweiligen `on-button`-Werten für Import und Senden. Kein zusätzlicher Markenakzent.
- **Destruktive Aktion:** `light-danger` / `dark-danger` für Löschen und sichtbare Fehler.

### Neutral

- **Arbeitsfläche:** `*-bg` für Hauptbereich, Menüs und Dialoge; `*-side` für Navigation und Codeblöcke.
- **Sekundäre Fläche:** `*-surface` für Felder, Codekopf und Inline-Code; `*-hover` für Navigation und Auswahl.
- **Gespräch:** `*-bubble` und `*-composer` für Nutzernachricht und Eingabe.
- **Lesbarkeit:** `*-text`, `*-muted` und `*-line` für Inhalt, Nebeninformation und Trennlinien.

Fokus verwendet `focus`, der lokale Statuspunkt `local-status`. Der KD-Avatar behält `avatar-bg` / `avatar-text` in beiden Themes. Diese kleinen Funktionsfarben bilden keine zweite dekorative Palette. Beim ersten Start wird die Betriebssystempräferenz übernommen. Danach schalten die beiden synchronisierten Toggle-Switches zwischen Hell und Dunkel um.

**The Pinned Reference Rule.** Die freigegebene ChatGPT-artige Gestaltung mit eigener KD-Marke bleibt die visuelle Autorität.

## Typography

**Display Font / Body Font:** Segoe UI, Arial, sans-serif; lokal aus Windows.
**Code Font:** Consolas, Cascadia Code, monospace.

Die Schrift unterstützt die vertraute Chat-Anmutung. Navigation bleibt kompakt, Antworten erhalten mehr Grösse und Zeilenabstand. Die Frontmatter enthält die beobachteten Rollen; kein künstliches Skalensystem wird ergänzt.

### Hierarchy

- **Display:** Erststartüberschrift; die grössere KD-Wortmarke ist separat (43px, Gewicht 750, Laufweite -4px).
- **Headline:** Leerer Chat. Bei mittleren und schmalen Fenstern reduziert sich die Grösse (26px / 24px).
- **Title:** Modellwahl und Dialogtitel; Modellversion ist leichter und kleiner (16px, Gewicht 400).
- **Body:** Navigation, Felder und Dialogtext. Gesprächsinhalt verwendet die Rolle `message`; gerendertes Markdown erhöht den Zeilenabstand auf (1.8).
- **Label:** Chatgruppen und Menülabel. Herkunft und Nebenmetadaten können kleiner sein (10–11px).
- **Code:** Codeblöcke mit eigener Kopfzeile und Kopieraktion; kein Syntax-Highlighting.

**The Local Type Rule.** Die Segoe-Systemschrift ist Teil der festgelegten Referenz; keine externen Fonts ergänzen.

## Layout

Die App füllt das Fenster (100dvh). Standard ist ein Zweispaltenlayout: Sidebar (260px), flexibler Hauptbereich. Sidebar und Topbar haben einen kompakten Kopf (64px). Chats und Composer teilen eine zentrierte Inhaltsbreite (maximal 768px), mit seitlichem Abstand (28px). Ab einer Fensterbreite (1500px) steigt die maximale Inhaltsbreite auf (800px).

Bis (900px) wird die Sidebar schmaler (220px), der Hauptabstand sinkt auf (20px). Bis einschliesslich (720px) liegt die Sidebar als feste Ebene (260px) mit Scrim über dem Inhalt; die Grid-Spalte ist dann null. Der Gesprächsabstand wird (18px), der Composer-Abstand (16px). Die lokale Statusanzeige wird in dieser schmalen Ansicht verborgen. Eine eingeklappte Sidebar erhält auch oberhalb des Breakpoints keine Grid-Breite.

Nutzernachrichten sind rechtsbündig, maximal (78%) breit; schmal maximal (90%). Assistentenantworten liegen direkt auf der Hauptfläche. Der leere Chat stellt Überschrift und Composer um die obere Fenstermitte; ein laufendes Gespräch hat einen scrollenden Nachrichtenbereich und den Composer darunter. Dialoge sind maximal (640px) breit, maximal (85dvh) hoch und intern scrollbar; auf schmalen Fenstern wird die Aussenpolsterung reduziert.

## Elevation & Depth

Tonal gestaffelte Flächen bilden die Grundtiefe. Composer, Modellmenü, Dialog und Sidebar-Overlay erhalten gezielte Schatten. Exakte Schatten und Bewegungswerte stehen als Erweiterungen in [.impeccable/design.json](.impeccable/design.json), weil sie nicht zum Frontmatter-Komponentenschema gehören.

### Shadow Vocabulary

- **Menü / Fehlerhinweis:** themeabhängiger `--shadow`.
- **Composer:** sehr schwacher Kontaktschatten.
- **Dialog:** deutlicher Schatten hinter einer abgedunkelten Ebene.
- **Schmale Sidebar:** seitlicher Schatten und Scrim.

**The Selective Depth Rule.** Gesprächsflächen bleiben flach. Schatten kennzeichnen Composer, Menüs und modale Ebenen.

Bewegung bleibt kurz: Sidebar und Grid gleiten, Menüs und Nachrichten erscheinen mit kleinen Verschiebungen, Ladepunkt und Streamingcursor pulsieren. `prefers-reduced-motion: reduce` deaktiviert sämtliche Animationen und Übergänge und erzwingt automatisches Scrollverhalten.

## Shapes

Die Formensprache verbindet kleine gerundete Navigationsflächen mit deutlich weicheren Gesprächsformen. Die Frontmatter ist die normative Radiusliste: Controls, Felder, Code, Menüs, Dialoge, pillenförmige Aktionsbuttons, Nutzernachrichten und Composer haben eigene Rollen. Der Composer verwendet (26px); Sendebutton und Profilavatar sind kreisförmig. Feine Linien begrenzen Felder und getrennte Inhaltsbereiche. Der Composer hat zunächst eine transparente Kontur und zeigt bei Fokus eine thematische Trennlinie.

## Components

### Buttons

Hauptaktionen sind monochrom mit invertierter Schrift, sekundäre Aktionen haben eine feine Kontur, destruktive Aktionen verwenden die Fehlerfarbe. Die gemeinsamen Aktionsbuttons haben Padding (11px 18px), Zeilenhöhe (20px) und pillenförmige Rundung. Hover reduziert bei Hauptaktionen die Deckkraft; sekundäre Aktionen wechseln zur sekundären Fläche. Normale deaktivierte Buttons haben Deckkraft (.45), Senden (.25). Aktivierung verkleinert aktive Buttons leicht (scale .97). Sichtbarer Tastaturfokus verwendet eine Kontur (2px) mit Abstand (3px).

Iconbuttons sind standardmässig (34px) quadratisch; Nachrichtenaktionen sind kleiner (28px). Der Sendebutton ist (36px) gross und wechselt während der Generierung zum quadratischen Stoppsymbol. Die Symbole stammen aus den eingebetteten SVG-Pfaden in `src/ui.rs`.

### Inputs / Fields

Das normale Textfeld verwendet sekundäre Fläche, Kontur, Feldradius und Padding (12px). Suche ist in die Navigation integriert und besitzt keine umrahmte Eingabebox. Der Composer enthält eine transparente, automatisch wachsende Textarea mit Zeilenhöhe (25px), Minimalhöhe (35px) und Maximalhöhe (200px). Seine Aussenkontur zeigt den Fokus; die Textarea selbst unterdrückt ihre eigene Outline. Enter sendet, Shift+Enter fügt einen Absatz ein.

### Navigation

Sidebar-Zeilen verwenden kleine Rundung und Tonflächen für Hover und Auswahl. Chatnamen sind einzeilig mit Auslassungszeichen. Weitere Chataktionen erscheinen bei Hover, Tastaturfokus oder Auswahl. Das KD-Profil am Fuss öffnet Einstellungen. Der Modelltrigger im Hauptkopf öffnet ein begrenztes Menü mit Modellversion und Auswahlmarkierung; Modellwechsel sind bei laufender Generierung gesperrt.

### Conversation / Composer

Nutzernachrichten erhalten eine weich gerundete Tonfläche. Assistententext bleibt ohne Kartenrahmen. Markdown unterstützt Überschriften, Listen, Tabellen, Zitate und Codeblöcke mit Kopierkopf; horizontales Überlaufen bleibt innerhalb des jeweiligen Inhaltsbereichs. Unterbrochene und fehlgeschlagene Antworten bleiben sichtbar gekennzeichnet. Herkunft erscheint an der Antwort bei Hover.

Der Composer führt Text, Modellverweis und Senden/Stop in einer gemeinsamen Fläche zusammen. Die modellfreie Erstansicht zeigt eine zentrale Importaktion und die eigene KD-Wortmarke. Die freigegebene App enthält keine Voice-, Websuch-, Bildgenerierungs- oder Reasoning-Auswahl.

### Dialogs / Feedback

Modellverwaltung, Einstellungen, Umbenennen und Löschbestätigung verwenden dieselbe Dialoghülle. Modelle werden als getrennte Zeilen mit Metadaten und Aktionen gezeigt, ohne zusätzliche dekorative Karten. Fehler erscheinen als begrenzter Hinweis über dem unteren Inhalt mit eigener Schliessaktion. Diese Desktop-Komponente ist keine Smartphone-Benachrichtigung.

## Do's and Don'ts

### Do:
- Do die vorhandenen hellen und dunklen semantischen CSS-Variablen verwenden.
- Do Gesprächsinhalt und Composer gemeinsam zentrieren und ihre Breitenregeln erhalten.
- Do Tastaturfokus, reduzierte Bewegung und native semantische Bedienelemente erhalten.
- Do die eigene KD-Wortmarke, assets/kd.ico und die Inline-SVGs aus src/ui.rs verwenden.

### Don't:
- Don't OpenAI-Markenassets oder fremde Schriftdateien hinzufügen.
- Don't die festgelegte Chatoberfläche durch eine neue Designrichtung ersetzen.
- Don't visuelle Pixelgleichheit oder vollständige Klick-QA als nachgewiesen darstellen.
- Don't dekorative Karten, Farbverläufe oder zusätzliche Akzentfarben als bestehende Systemregeln erfinden.


## KD Desktop v1.0 – Navigation und Theme-Switch

App-Name und Modellwahl stehen getrennt im Kopfbereich. Modellversionen kennzeichnen nur das aktive Modell; die Anwendung führt ihre eigene Version v1.0.

Die Darstellungsauswahl ist ein semantischer Button mit `role=switch` und `aria-checked`. Oben rechts und in den Einstellungen steht derselbe 48 × 28 px grosse Toggle bereit. Die 22 px grosse runde Schaltfläche bewegt sich über 20 px; sie trägt das eigene Sonnen-/Mondsymbol. Der Track nutzt die vorhandenen semantischen Linien-/Textfarben. Die gespeicherte Auswahl wird lokal wiederhergestellt; beim ersten Start gilt die Systempräferenz. Der vorhandene Reduced-Motion-Stil gilt auch für den Switch.

Eine geöffnete Sidebar besitzt kein `inert`-Attribut. Nur die geschlossene Sidebar trägt `inert` mit leerem Wert. Damit bleiben unsichtbare Fokusziele gesperrt, ohne die offene Navigation durch `inert="false"` zu deaktivieren.

Gezielte automatisierte Navigation-/Theme-Tests mit denselben Dioxus-Komponenten und einem echten nativen WebView2 sind bei 1280 und 560 px bestanden. Sie prüfen gerenderte Attribute, Fokus, Hit-Test, Rust-Handler, Theme-Farben und SQLite-Persistenz. Eine vollständige visuelle Abnahme oder ein Pixelvergleich mit ChatGPT wird dadurch nicht behauptet.
