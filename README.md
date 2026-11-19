# Künstliche Dummheit (KD)

Lokales experimentelles Sprachmodell für kreative, offensichtlich falsche und absurde Antworten. **Aktueller Teststand: v0.4.1. Grammatik, Antwortlänge und konstante absurde Wirkung sind weiterhin unzureichend.**

## Schnell starten

- **KD Desktop v1.0:** [KD Desktop starten.cmd](<KD Desktop starten.cmd>) startet den letzten Desktop-Build. [Anleitung, Modellformat und Installer-Build](desktop/README.md). Windows-Installer ohne Gewichte; KD-Modell-ZIP separat importieren. Die App-Version ist unabhängig von der Modellversion; die unveränderten v0.4.1-Gewichte bleiben nutzbar.
- **KD v0.6 trainieren:** [RTX-4090-Übergabe mit eigenen JSONL und Nachtprofil](uebergabe/KD-v0.6-RTX4090-20261116-221506Z/README.md). Qwen2.5-7B-Instruct, lokales QLoRA, Validierung und Early Stopping; noch kein trainiertes v0.6-Modell. [Prüfbericht](docs/reports/KD-v0.6-TRAININGSUEBERGABE.md).
- [KD Webseite starten.cmd](<KD Webseite starten.cmd>) öffnet die neue lokale Webseite: Modell-Chats und Qwen3.5-9B/KD-Dialoge mit eigenem Ziel. [Benutzung und Voraussetzungen](web/README.md).
- [KD starten.cmd](<KD starten.cmd>) öffnet die aktuelle KD im Terminal.
- [Portable v0.4.1 als ZIP, etwa 732 MB](builds/KD-0.4.1-20261106-103336.zip): vollständig entpacken und `KD.exe` starten. CPU-Betrieb, ohne Internet, Python oder zusätzliche Installation beim Empfänger.
- [v0.4 zum Vergleich](builds/KD-0.4-20261106-095836.zip) und [weitere Starter](launchers/) bleiben erhalten.
- [Original starten.cmd](<Original starten.cmd>) startet das ursprüngliche Qwen2.5-0.5B über die lokale Python-Entwicklungsumgebung.

Im Terminal: `/hilfe`, `/neu`, `/kurz`, `/lang`, `/speichern DATEI.txt`, `/ende`. Strg+C hält eine laufende Antwort an.

## Für Entwickler und andere Agenten

**Zuerst [AGENTS.md](AGENTS.md), dann [ARCHITECTURE.md](ARCHITECTURE.md) lesen.** Dort stehen Projektziel, Nutzerregeln, tatsächlicher Entwicklungsstand, Dateiplan, geprüfte Arbeitsbefehle und offene Probleme.

| Gesucht | Ort |
|---|---|
| Aktueller Bericht und Grenzen | [v0.4.1](docs/reports/KD-v0.4.1.md) |
| Aktuelle Desktop-App, Installer und Prüfungen | [KD Desktop v1.0](docs/reports/KD-Desktop-v1.0.md) |
| Frühere Desktop-App und damalige Prüfgrenzen | [KD Desktop 0.5.0](docs/reports/KD-Desktop-v0.5.0.md) |
| Tatsächliche Original-/KD-Antworten | [Vergleich](runs-sft/v041-qwen3-long/evaluation/VERGLEICH.md) |
| Trainingsdaten | `data/muster_antworten_bereinigt.jsonl` |
| Training, Export, Prüfung, Build | `scripts/` |
| Konsolenanwendung / Engine | `terminal/`, `app/LocalEngine.cs` |
| Modelle / auslieferbare Versionen | `models/`, `builds/` |
| Berichte / frühere Versuche | `docs/reports/`, `runs*/`, `validation*/` |
| Frühere Konsolenlogs | `logs/history/` |
| Originalprüfsummen / Hardwareaufnahme | `metadata/references/`, `metadata/hardware/` |

Die Entwicklung begann mit direkten Gewichtseingriffen in Qwen2.5. Auf späteren ausdrücklichen Auftrag kam LoRA-Training auf den Nutzerbeispielen hinzu. Die aktuelle Qwen3-Variante enthält die gelernten Änderungen in eigenständigen Modellgewichten; ein Dummheits-Systemprompt oder eine Antworttabelle wird nicht verwendet. Die frühere WPF-Oberfläche ist ein Altstand. Die portable Anwendung bleibt ein Terminalprogramm; die ausdrücklich beauftragte lokale Webseite ergänzt sie.

Alte Builds und Originalmodelle werden nicht gelöscht oder überschrieben. Die Ordnerstruktur wurde am 6. November 2026 aufgeräumt; [Umzugsliste und Sicherungshinweise](metadata/reorganization/20261106-104237/moves.json) dokumentieren die früheren Dateipfade. Historische Befehle in alten Laufkopien/Logs können noch diese alten Pfade enthalten.
# KD-K-nstliche-Dummheit
