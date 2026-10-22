# SolidCraft – Chatlauf und nahtlose Projektübergabe

> **Stand: 22. Oktober 2026, nach erfolgreich bestandenem Testlauf #19.**
>
> **Dieses Dokument ist der fortlaufende Kontext für einen neuen Chat.** Es fasst den Projektverlauf, die getroffenen Entscheidungen, die tatsächlich implementierten Funktionen, bestätigte Tests, Grenzen, nächste Arbeitsschritte und Zugriffsbesonderheiten zusammen. Es ist **keine wörtliche Abschrift** jedes Chatbeitrags. Nach jedem größeren Entwicklungsschritt und ausgewerteten GitHub-Test bitte aktualisieren.
>
> **Wichtig:** Der tatsächlich letzte verifizierte Quellcode der getesteten Zeichnungsfunktionen war Commit \`3c665607fa3a045a9293ee0c1ef330c4651e5286\` auf \`feature/browser-automation\` in Run #19. Änderungen ausschließlich an Dokumentationsdateien danach benötigen keinen erneuten CAD-Test.

## 1. Projekt und Zusammenarbeit

- **Projekt:** GitHub SolidCraft, ein CAD-System; wir entwickeln insbesondere eine professionelle, manuell gestaltbare und mit dem 3D-Modell verknüpfte technische **2D-Fertigungszeichnung**.
- **Nutzer-Fork:** https://github.com/Benny1496/solidcraft
- **Upstream-Repository:** https://github.com/tangency-labs/solidcraft — **nicht** selbstständig ändern oder dorthin pushen.
- **Arbeitsbranch:** \`feature/browser-automation\` im Fork. Der Branch \`main\` ist **nicht** unser Zeichnungs-Entwicklungsbranch.
- **Arbeitsmethode:** Quellcode, Tests, Workflow und Dokumentation direkt über GitHub/GitHub-Connector ändern; der Nutzer soll **nicht lokal am PC** bauen, installieren oder testen müssen.
- **Sprache:** Deutsch, technisch konkret, übersichtlich; gerne mit motivierender, aber sachlicher Rückmeldung.
- **Arbeitsstil des Nutzers:** Möglichst große **gebündelte automatisierte Testblöcke**, damit in einem GitHub-Lauf viele Funktionen geprüft werden. **Nicht beim ersten Fehler stoppen**, sondern sämtliche unabhängigen Testgruppen abarbeiten. Am Ende jeden Fehler sammeln und den Gesamtlauf **rot** markieren, wenn mindestens eine Prüfung fehlschlägt. Nicht Ergebnisse als erfolgreich bezeichnen, bevor GitHub-Protokolle dies bestätigen.
- **GitHub-Workflow:** https://github.com/Benny1496/solidcraft/actions/workflows/solidcraft-browser-test.yml
- **Workflowname:** \`SolidCraft Browser Smoke (manual)\`
- **Start:** Nutzer klickt auf GitHub **Actions → SolidCraft Browser Smoke (manual) → Run workflow** und wählt ausdrücklich \`feature/browser-automation\`. Nicht \`main\`; einen neuen Lauf starten statt bei Quellcodeänderungen einen alten Commit nur „rerun“ zu lassen.
- **Wichtige Randbedingung:** In dieser Zusammenarbeit konnte der GitHub-Connector den manuellen Workflow nicht selbst auslösen; daher startet der Nutzer den Test und der Assistent **prüft GitHub-Status, Jobs und Logs direkt über die GitHub-Verbindung**. Keine Screenshots oder lokale Konsole nötig.
- **Wunsch des Nutzers:** Diese \`Chatlauf.md\` bei weiteren Änderungen fortführen und bei jedem neuen Chat **als erste Projektkontextquelle lesen**; zusätzlich \`docs/experimental-linked-drawing.md\` für technische Details.

## 2. Produktziel: gewünschte Arbeitsweise der Fertigungszeichnung

Ziel ist eine nutzbare technische 2D-Zeichnung, etwa wie in Fusion 360:

1. Aus einem vorhandenen 3D-CAD-Modell eine neue 2D-Zeichnung erstellen bzw. die Zeichnung später wieder öffnen.
2. Der Nutzer wählt die gewünschten Ansichten und setzt anschließend **selbst**, wo welche Maße auf der Zeichnung liegen. **Nicht** automatisch sämtliche Skizzen- und Modellmaße ungefiltert in die Zeichnung kopieren.
3. Gewünschte Bemaßungen: Außenmaße, Bohrungs-Ø, Mittelpunkt- und Kantenabstände, später auch Radien, Winkel, Tiefe, Fasen, Toleranzen und weitere Fertigungsangaben.
4. Die einmal gesetzten Maße gehören eindeutig zu bestimmten 3D-Flächen/-Kanten/-Merkmalen. Bei Änderungen am 3D-Bauteil sollen Werte und Positionen so weit wie sicher möglich nachgeführt werden.
5. **Ganz wichtig:** 3D-Modelländerungen dürfen nicht die 3D-Arbeit durch ein ungefragtes Fenster stören. Beim **Öffnen/Wechseln in die 2D-Zeichnung** wird bei erkanntem 3D-Unterschied eine **ausdrückliche Bestätigung** verlangt: „Zeichnung aktualisieren“ oder „Alte Zeichnung beibehalten“. Die gespeicherte Zeichnung darf nicht still verändert werden.
6. Eine nicht mehr sicher identifizierbare Geometrie darf **niemals** anhand ihrer Listenposition oder nur einer ähnlichen Bohrungsposition stillschweigend einer anderen Bemaßung zugeordnet werden. Das Maß bleibt erkennbar **ungeklärt**.
7. Langfristig: richtige projizierte Außenkonturen, mehrere Ansichten, normgerechte Maßlinien, Zeichnungsrahmen und Schriftfeld, Maßstäbe, geeignete Druck-/PDF-/DXF-Ausgabe; funktionierende echte Dokumentstruktur statt bloßer provisorischer Metadaten.

## 3. Was bereits implementiert und bis Run #19 getestet ist

### 3.1 Zeichen-Vorschau, Geometrie und 3D-Änderungen

- Rust-Backend: \`crates/engine/src/cmd/drawing_preview.rs\`.
- Befehl \`drawing.preview\` erstellt aktuell eine **experimentelle XY-Draufsicht** eines ausgewählten 3D-Körpers.
- Länge/Breite kommen aus der gemessenen 3D-Bounding-Box. Die Außenkontur wird aktuell **nur als Rechteck-Hüllrahmen** dargestellt, **nicht** als exakte projizierte Silhouette.
- Aus tatsächlich gemessenen z-achsigen Zylinderflächen werden Kreise, Mittelpunkte, Durchmesser sowie X-Abstände von der linken und Y-Abstände von der unteren Hüllrahmenkante berechnet.
- Zylinderflächen **nicht** pauschal als echte Bohrungen zertifizieren: Auch andere zylindrische Geometrien können enthalten sein.
- Antwort enthält ein versioniertes Zeichnungsobjekt \`solidcraft-drawing-preview/1\` mit \`source\`, \`geometry\`, \`dimensions\`, \`svg\`, \`label_positions\`, \`annotations\` und Grenzen/Hinweisen.
- Statussystem: \`created\`, \`current\`, \`stale\` (mit \`requires_confirmation: true\` und getrennter \`proposed\`-Zeichnung), \`updated\` nach \`accept: true\`.
- Die Vergleiche der gemessenen Quellen sind **numerisch**, nicht auf JSON-Integer/Float-Typidentität angewiesen. Parameteränderung, Modell-Undo/Redo, Bohrungsänderung und gespeicherte Datei wurden geprüft.

### 3.2 Echte auswählbare, verknüpfte Bemaßungen – im Prototyp

- Zusätzlicher Befehl \`drawing.annotation.add\`: benötigte Parameter \`body\`, \`previous\`, \`face_name\` sowie \`kind\` = \`diameter\`, \`x_from_left\` oder \`y_from_bottom\`; optional \`position_mm: [x,y]\`.
- **Nur** Zylinderflächen mit eindeutig zuordenbarem, aus der 3D-Modellhistorie **verifiziertem** \`model.faces\`-Namen (\`exact: true\`) dürfen eine solche Bemaßung erhalten.
- Pro gemessenem Zylinder werden potenzielle verifizierte Flächennamen und gegebenenfalls ein \`eligible_face_name\` bereitgestellt. Bei mehreren gleichnamigen Zylindern wird eine unklare Zuordnung nicht zugelassen.
- Jede vom Nutzer gesetzte Maßangabe besitzt eine eigene Kennung (\`linked_cyl_N\`), ihren Flächennamen, Art, Messwert, Mittelpunkt und eine separat gespeicherte, verschiebbare Position.
- Werte werden beim erneuten Aufruf anhand der **verifizierten Flächenkennung** nachgeführt; **nie** anhand der Position in einer Zylinderliste.
- Fehlt die 3D-Referenz nach einer Änderung, bleibt die Annotation mit \`missing_reference\` bestehen, besitzt **keinen gültigen aktuellen Wert** (\`value_mm: null\`) und behält einen ausdrücklich historischen \`last_value_mm\`. Nicht auf eine zufällig ähnliche Bohrung umhängen.
- Weitere Regeln: Duplikate pro Fläche und Maßart sowie ungültige Eingaben ablehnen; bei offenem Änderungsdialog keine neuen Bemaßungen still anlegen.
- Zeichnungs-SVG zeigt die aktiv gesetzten Ø-, X- und Y-Maße samt Führungslinien. Ungelöste Referenzen werden im SVG mit \`?\` gekennzeichnet.

### 3.3 Echte 2D-Bedienoberfläche (egui)

- Code: \`crates/ui-egui/src/drawing_ui.rs\`; eingebunden in die SolidCraft-Bedienoberfläche im Bereich **INSPECT → DRAWING**.
- Fenster: **„2D-Zeichnung (Testversion)”**, mit Auswahl des 3D-Körpers, vorläufiger Draufsicht, Maßtexten, Kreiszentren und Bohrungsmaß-Tabelle.
- Nutzereingriffe: Zeichnung im Modell speichern; Außenmaßtexte per Maus verschieben; bei eindeutig verifizierten Zylindern **„Ø setzen“**, **„X setzen“**, **„Y setzen“** anklicken; neu angelegte Maßtexte unabhängig verschieben und dauerhaft speichern.
- Bei 3D-Änderung den bisherigen gespeicherten Stand getrennt vom Aktualisierungsvorschlag anzeigen. **„Zeichnung aktualisieren“** akzeptiert die neue Geometrie; **„Alte Zeichnung beibehalten“** lässt das gespeicherte Objekt unverändert.
- Ältere gespeicherte Zeichnungssnapshots ohne neue Maßtabellen werden weiterhin angezeigt. Vorheriger Fehler beim Zurücksetzen von per Maus verschobenen Maßtexten wurde korrigiert.
- Persistenz über \`document.metadata.set\` unter dem Schlüssel \`org.solidcraft.experimental_drawing\`. **Noch kein natives Drawing-Dokument**; es handelt sich um ausdrücklich experimentelle, versionierte Informationsmetadaten.
- Tatsächliches Speichern und Öffnen einer \`.solidcraft\`-Datei inklusive Zeichnungsstand und späterer 3D-Änderung wurde automatisiert getestet.

## 4. Testsystem – dauerhafte Vereinbarung

Workflowdatei: \`.github/workflows/solidcraft-browser-test.yml\`

Der Workflow hat fünf unabhängige Prüfgruppen:
1. JavaScript-Syntax aller \`xtask/src/browser_*_test.js\`.
2. SolidCraft-WebAssembly-Build (\`trunk build --release --public-url ./\`).
3. Chrome-Browser-API plus CAD-Szenarien über \`cargo xtask web-api-test --site dist/web\`.
4. Echte egui-Interaktionstests: \`cargo test -p solidcraft-ui-egui drawing_ui::tests --lib -- --nocapture\`.
5. Rust-Zeichnungs-/Geometrietests: \`cargo test -p solidcraft-engine drawing_preview --lib -- --nocapture\`.

**Dauerhaft beibehalten:**
- \`continue-on-error: true\` für unabhängige Prüfungen. Auch wenn eine fehlschlägt, laufen die anderen weiter (\`if: !cancelled()\`).
- Browser-Checks nur ausführen, wenn der notwendige WASM-Build erfolgreich ist; ein übersprungener abhängiger Test gilt **nicht** als bestanden.
- Der Browser-Runner \`xtask/src/web_offline_test.rs\` fängt Fehler aller einzelnen Skripte ab, protokolliert sie und führt die anderen Szenarien weiter aus, sofern Chrome/CDP selbst verfügbar bleibt.
- Am Ende erzeugt GitHub eine **Step Summary** mit allen fünf Ergebnissen. Der Gesamtlauf wird **rot**, wenn auch nur eine Gruppe fehlschlug oder übersprungen wurde.
- Eine testbare Web-App wird auch bei Fehlern als kurzlebiges GitHub-Artefakt gespeichert.
- Bei fehlgeschlagenen Tests **GitHub-Joblogs prüfen**, Quellcode im Fork korrigieren, neuen manuellen Lauf auf dem aktuellen Branch auslösen lassen.
- Umfang nach Run #19: **25 Browserprüfungen** (23 CAD-Szenarien + API-Smoke + Referenzplatte), **11 egui-GUI-Tests**, **14 Rust-Kerntests**, zusammen **50/50 bestanden**.

## 5. Chronologie wichtiger GitHub-Testläufe

| Lauf | Ergebnis | Erkenntnis |
| --- | --- | --- |
| [#12](https://github.com/Benny1496/solidcraft/actions/runs/39656227739) | Erfolg | Frühe Teststufe |
| [#13](https://github.com/Benny1496/solidcraft/actions/runs/39661273203) | Erfolg | Frühe gespeicherte Zeichnung, echte GUI-Mausinteraktionen |
| [#14](https://github.com/Benny1496/solidcraft/actions/runs/39669359298) | Fehler | 15/15 CAD und 6/6 GUI erfolgreich, 1 Rust-Assertion verglich dieselben Koordinaten als JSON-Int statt Float |
| [#15](https://github.com/Benny1496/solidcraft/actions/runs/39675912802) | **Erfolg** | Nach Korrektur 28 Testfälle plus API-Smoke erfolgreich |
| [#16](https://github.com/Benny1496/solidcraft/actions/runs/39719664411) | Abgebrochen | Versehentlich auf \`main\` gestartet; keine Quellcodeänderung |
| [#17](https://github.com/Benny1496/solidcraft/actions/runs/39719878000) | Fehler | 19/19 CAD und 8/8 Kern erfolgreich; 2 GUI-Regressionen bei dauerhaftem Verschieben der X-/Y-Texte; alle unabhängigen Prüfungen liefen trotzdem weiter |
| [#18](https://github.com/Benny1496/solidcraft/actions/runs/39724896543) | **Erfolg** | **38/38**: 21 Browserprüfungen, 9 GUI, 8 Kern; Testorganisation mit abschließender Gesamtauswertung bestätigt |
| [#19](https://github.com/Benny1496/solidcraft/actions/runs/39731900004) | **Erfolg** | **50/50**: 25/25 Browser, 11/11 GUI, 14/14 Rust. Neue Flächenreferenz-Bemaßungen Ø/X/Y, Löschfall, ungültige Eingaben, Mausverschiebung und echter Save/Open-Durchlauf alle erfolgreich; **0 fehlerhafte Testgruppen** |

**Stand nach #19:** SHA \`3c665607fa3a045a9293ee0c1ef330c4651e5286\`, Run am 22.10.2026 abgeschlossen; der gesamte Workflow inklusive Syntaxprüfung, WASM-Build, Browser, GUI, Rust und Abschlusssummary war **grün**. Danach ist die Dokumentationsdatei \`Chatlauf.md\` hinzugekommen. Alle Prüfergebnisse gelten für die genannte getestete SHA, nicht pauschal für spätere Codeänderungen.

## 6. Verzeichnis wichtiger Projektdateien

- \`Chatlauf.md\` (**diese Datei**): übergreifender chronologischer Projekt-/Chatkontext, Neu-Chat-Übergabe und verbindliche Arbeitsvereinbarungen.
- \`docs/experimental-linked-drawing.md\`: detaillierter Entwicklungsbericht, technische Grenzen, Testerweiterungen und einzelne Fehlerkorrekturen.
- \`crates/engine/src/cmd/drawing_preview.rs\`: Rust-Backend der experimentellen Zeichnung mit Messung, Signatur, \`drawing.preview\`, \`drawing.annotation.add\`, Referenzverwaltung und Rust-Tests.
- \`crates/ui-egui/src/drawing_ui.rs\`: Zeichnungsfenster, Popup, Mausverschiebung, Auswahl und GUI-Tests.
- \`xtask/src/web_offline_test.rs\`: Browser-/CDP-Testharness; Liste der inkludierten CAD-JavaScript-Szenarien, deren Gesamtprotokoll und Auswertung.
- \`xtask/src/browser_drawing_linked_diameter_test.js\`: Verknüpftes Ø-Maß mit geänderter Modell-Ø/Bohrungsposition und SVG-Anzeige.
- \`xtask/src/browser_drawing_linked_dimensions_test.js\`: Ø-, X- und Y-Anmerkungen derselben verifizierten Fläche sowie separate Positionen/Duplikatschutz.
- \`xtask/src/browser_drawing_linked_missing_test.js\`: Entfernen einer Bohrung → Maß mit fehlender Referenz bleibt ausdrücklich ungeklärt.
- \`xtask/src/browser_drawing_linked_rejections_test.js\`: Falsche Flächennamen, fehlerhafte Maße, doppelte Einträge, ausstehende Modelländerungen.
- \`xtask/src/browser_drawing_dim_*_test.js\`: Geometriedatenübertragung, versetzte Ursprünge, Bohrungsanzahl/Undo/Redo, Mehrlochplatten.
- \`.github/workflows/solidcraft-browser-test.yml\`: manuell gestarteter, gebündelter, bewusst nicht frühzeitig abbrechender CI-Workflow.

## 7. Was ausdrücklich noch offen ist

Trotz des grünen Testlaufs ist die 2D-Fertigungszeichnung **noch experimentell**. Priorisierte nächste Schritte:

1. **Natürliche Direktselektion** einer Bohrung/Kante unmittelbar in der Zeichnungsansicht statt nur „Ø/X/Y setzen“ in der Zylinder-Tabelle; dabei sichere Referenzauswahl und sichtbares Hervorheben.
2. **Mehr vollständige Maßtypen:** tatsächliche Mitte-zu-Mitte-Abstände, Radius, Winkel, Kantenabstände aus frei wählbaren Bezugselementen, Bohrungstiefen und ggf. Durchmesser/Tiefen-Spezifika von Senk- und Gewindebohrungen.
3. **Bearbeitung/Löschen gesetzter Maßangaben**, Mehrfachauswahl und feinere Drag-/Label-Platzierung mit Kollisionserkennung.
4. **Verlässliche 2D-Projektion echter Konturen**: Rundungen, Fasen, Taschen, komplexe 3D-Bauteile statt bloßem Rechteck-Hüllrahmen; verdeckte Kanten und weitere Ansichten.
5. **Robuste persistente Feature-/Face-Referenzlogik** für Split/Merge der Topologie, zusammengesetzte Körper und importierte Geometrien. Fehlende oder ungesicherte Namen weiterhin **nicht** automatisch neu zuordnen.
6. Zeichnungsdokumente/Blätter, einstellbare Maßstäbe, Rahmen, Schriftfeld, Normangaben, Toleranzen, Druck-/PDF-/DXF-Ausgabe.
7. Änderungen am 3D-Modell gemäß Nutzerwunsch bei Rückkehr zur 2D-Ansicht zur Bestätigung anbieten, ohne aufdringliche Modaldialoge während der 3D-Arbeit.
8. Weiterhin große, logisch unabhängige **nicht-fail-fast** Testbündel vorsehen, inklusive Negativfälle, Datei-Roundtrip und echter egui-Interaktionen.

### Sicherheits- und Wahrheitsgrundsätze

- Ein Face-Name wird **nur dann** zum Anker einer Fertigungsbemaßung, wenn er nachweislich historiesicher und eindeutig ist.
- **Keine** übertriebene Behauptung, die Rechteck-Bounding-Box sei eine tatsächliche B-Rep-Projektion oder alle erkannten Kreise seien Löcher.
- Nicht den GitHub-Workflow manuell auf \`main\` auslösen; nicht aus dem Erfolg eines alten Commits auf einen neuen schließen.
- Keine Ergebnisse, Testanzahl, Dokumentationssync oder Berechtigungen erfinden.

## 8. Home Assistant: freigegebener Projektordner und Dokumentationspflege

- **Bestätigter HA-Ordner:** `/share/GitHub SolidCraft/`
- **Dateien:** `/share/GitHub SolidCraft/Chatlauf.md` (dauerhafter Projekt-/Chatverlauf) und `/share/GitHub SolidCraft/PROJEKTSTATUS.md` (technischer Projektstatus).
- **Zugriff wiederhergestellt und verifiziert am 22.10.2026:** Die HA-Verbindung konnte den freigegebenen Ordner auflisten, `Chatlauf.md` als textidentische Kopie der GitHub-Datei schreiben und zurücklesen. Die bestehende `PROJEKTSTATUS.md` wurde erhalten, um einen **Nachtrag zu GitHub-Testlauf #19 mit 50/50 grünen Prüfungen** und den jüngsten Zeichnungsfunktionen erweitert, erfolgreich überschrieben und vollständig gegengeprüft.
- **Explizite Erlaubnis des Nutzers vom 22.10.2026:** Der Assistent darf die beiden genannten Markdown-Projektdokumente im freigegebenen HA-Ordner **bei der laufenden Projektarbeit jederzeit ohne vorherige Rückfrage lesen und aktualisieren**. Diese Erlaubnis bezieht sich auf die Projektdokumentation, **nicht** auf andere Home-Assistant-Systemdateien, Steuerungen oder Einstellungen. Die Verbindung muss weiterhin technisch verfügbar und für die Pfade freigeschaltet sein.
- **Historischer Kontext:** Ein älterer Schreibversuch wurde abgewiesen. Danach wurde der HA-Ordner neu freigegeben, die Verbindung erfolgreich geprüft und die aktuelle ausdrückliche Erlaubnis erteilt. Vermeide weitere Zugriffe nur, wenn die Verbindung **erneut** verweigert wird; in diesem Fall die aktuelle Sperre respektieren.
- **Dauerhafte Arbeitsvereinbarung:** Nach wesentlichen CAD-Entwicklungsschritten und nach tatsächlich bestätigten GitHub-Testläufen `Chatlauf.md` und `PROJEKTSTATUS.md` im freigegebenen HA-Ordner aktualisieren; die GitHub-Dokumente `Chatlauf.md` und `docs/experimental-linked-drawing.md` ebenfalls auf dem Laufenden halten. Historie nicht löschen, Ergänzungen nachprüfbar kennzeichnen und Schreib-/Leseerfolg prüfen.
- **Kein Hintergrunddienst:** Die Erlaubnis gestattet Updates im Rahmen unserer aktiven Arbeitsaufträge; sie startet keine eigenständigen automatischen, zeitversetzten Schreibvorgänge.

## 9. Anleitung für den nächsten Chat / den ausführenden Assistenten

Wenn ein neuer Chat beginnt und auf dieses Projekt Bezug genommen wird:

1. Als Erstes **\`/share/GitHub SolidCraft/Chatlauf.md\`** und **\`/share/GitHub SolidCraft/PROJEKTSTATUS.md\`** im freigegebenen HA-Ordner lesen; bei technischen Fragen ergänzend **\`docs/experimental-linked-drawing.md\`** im GitHub-Fork auf Branch \`feature/browser-automation\`. GitHub-\`Chatlauf.md\` ist Sicherungskopie.
2. Aktuelle Head-SHA, letzte GitHub-Action und Testprotokolle prüfen, insbesondere falls der Nutzer einen Lauf gestartet oder einen Fehler gemeldet hat.
3. Als zuletzt **verifizierten** Stand #19 mit 50/50 grünen Prüfungen verwenden. Nicht versehentlich erneut einen bereits behobenen Rückschritt beginnen.
4. Gemeinsam mit dem Nutzer die nächste 2D-Zeichnungsfunktion planen oder – wenn die Aufgabenstellung klar ist – direkt im GitHub-Branch implementieren. Immer den **großen, unabhängigen Testblock** beibehalten/erweitern.
5. Nach eigenem Code-Commit den Nutzer um einen neuen **manuellen** Workflow-Lauf auf dem korrekten Branch bitten; Logs selbst abrufen, Fehler gesammelt korrigieren.
6. **Nach bestätigtem Testergebnis beide HA-Dateien (\`Chatlauf.md\`, \`PROJEKTSTATUS.md\`) und beide GitHub-Dokumente (\`Chatlauf.md\`, \`docs/experimental-linked-drawing.md\`) aktualisieren.** Tests nicht als grün deklarieren, bevor sie bestanden wurden.
7. Die am 22.10.2026 ausdrücklich erteilte Dokumentations-Schreibfreigabe respektieren und nutzen, solange der HA-Ordner technisch verfügbar bleibt. Bei einer neuen tatsächlichen Zugriffsverweigerung nicht eigenmächtig Sperren umgehen.

**Kurzfassung zum Copy/Paste beim Start eines neuen Chats:**

> „Wir setzen GitHub SolidCraft fort. Bitte lies zuerst die Datei \`Chatlauf.md\` im Repo \`Benny1496/solidcraft\` auf \`feature/browser-automation\` und die Datei \`docs/experimental-linked-drawing.md\`. Halte die bisherigen Entscheidungen ein, prüfe den neuesten GitHub-Teststatus und arbeite ausschließlich direkt auf GitHub weiter. Nutze wieder große Testpakete mit vollständiger Fehlerauswertung; nach jedem erfolgreichen Durchlauf die Dokumentation fortführen. Die freigegebenen HA-Projektdateien im Rahmen unserer Arbeit ohne erneute Rückfrage aktuell halten.“

---

*Diese Datei dient bewusst als dauerhafter, praktischer Projektverlauf. Bei neuen Entwicklungen die vorhandenen bestätigten Ergebnisse **ergänzen statt überschreiben**. Immer aktuellen Teststatus, SHA und offene Einschränkungen auseinanderhalten.*
