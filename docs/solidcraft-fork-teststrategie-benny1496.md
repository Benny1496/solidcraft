# SolidCraft-Fork: Testbetrieb und Upstream-Synchronisierung (ab 02.11.2026)

## Aktuelle Regel: SolidCraft-Tests direkt im eigenen Fork

GitHub Actions sind für `Benny1496/solidcraft` wieder ausführbar. **Das ist technisch bestätigt**, nicht nur anhand der Billing-Einstellungen:

- Mini-Diagnose: [SolidCraft Fork Actions Availability Probe](https://github.com/Benny1496/solidcraft/actions/runs/41408307381), `completed/success` am 02.11.2026, Job `125155288391`.
- Originaljoblog meldet `SOLIDCRAFT_FORK_ACTIONS_PROBE=PASS`, `repository=Benny1496/solidcraft`, `runner_os=Linux`.
- Diagnose-Branch: `diagnostics/actions-availability-20261102`. Änderungen nur auf diesem Branch, nicht an PR #8288.
- Eine offizielle Mitteilung von GitHub Support über die Aufhebung wurde **nicht** erhalten; die erfolgreiche Ausführung ist der belastbare Nachweis.

**Ab jetzt werden alle neu geplanten SolidCraft-Tests primär in `Benny1496/solidcraft` ausgeführt.**
Das bisher genutzte Repository `Benny1496/ESP32-Victron-SolarSense-Bridge` bleibt als **historisches Archiv und Fallback** erhalten. Neue SolidCraft-Workflows nicht mehr dort anlegen, außer wenn eine neue technische Sperre den Fork erneut betrifft. Bereits durchgeführte externe Testläufe behalten ihre Originalbelege und URLs.

## Ablauf für künftige Tests

1. Ziel-Commit von `tangency-labs/solidcraft`, einem PR oder `Benny1496/solidcraft` **vollständig per SHA festlegen**; im Ergebnis nennen, welcher Branch und Quellstand tatsächlich getestet wurde. Upstream-Draft-PRs und unseren eigenen PR nicht mit `main` verwechseln.
2. **Kleine, gezielte native Tests** statt Wiederholungen der immer gleichen großen Matrizen. Gesamt-Workflow höchstens **90 min**, für Build/Testgruppe maximal **75 min**, möglichst einmal kompilieren.
3. Testabläufe auf einem dafür vorgesehenen Branch im Fork erstellen; Produktionsfunktionen oder PR #8288 nicht durch Test-Hilfscode verändern. Erweiterungen ggf. nur temporär auf dem Runner und ausdrücklich als `runner-only` kennzeichnen.
4. Tatsächliche GitHub-Actions-Ergebnisse **anhand Originaljoblog** und exakter Testzahlen auswerten. `queued`, `in_progress`, Preflight-Fehler und Rust-Testfehler unterscheiden; keine Erfolgszahlen ohne nachweislich bestandene Tests.
5. Pro Lauf einen Bericht `Testberichte/Test_NN/Zusammenfassung.md` und ein möglichst unverändertes Originaljoblog unter `/share/GitHub SolidCraft/Testberichte/Test_NN/` auf dem Home-Assistant-Share sichern, **sofern der Zugriff autorisiert ist**. GitHub-Artifact-ZIPs nur als auf HA gespeichert bezeichnen, wenn ihr Binärdownload tatsächlich erfolgt ist. Bei fehlendem HA-Zugriff das nicht als erfolgreich dokumentieren.
6. Nach dem Lauf die eigentliche GitHub-Actions-URL aus dem Fork und die geprüfte Commit-SHA in den Bericht aufnehmen. Bestehende Berichte und Sicherungen nicht überschreiben.

## Historische Ergebnisse (nicht neu laufen lassen)

- Test #51, externer ESP32-Runner: Windows, **71/71** native Rust-Tests, PR #8288 `c86b7a4c2`.
- Test #52, externer ESP32-Runner: Ubuntu Linux, **71/71** auf demselben PR-Commit.
- Test #53, externer ESP32-Runner: Windows, **73/73** (71 bestehende + 2 nur temporär im Runner ergänzte Tests).
- Test #54, externer ESP32-Runner: Linux, **7/7** Drawing-Export-Tests inkl. W23-2/W23-5, auf CamWights PR #10881 `c3f8cf3`.
- CamWights eigene Windows-Gegenprüfung auf PR #8288: **43/43** relevante Tests, nicht die komplette Engine-Suite.

Alle Zahlen gelten **nur für die jeweiligen Commitstände und Testgruppen**.

## Sync des eigenen Forks

- `main` soll den neuesten Upstream-Stand von `tangency-labs/solidcraft:main` enthalten und die eigene `.github/workflows/solidcraft-browser-test.yml` behalten.
- Die alte `main`-Revision vom 02.11.2026 ist unter `backup/main-before-upstream-sync-20261102` gesichert.
- Eine reine Synchronisation soll keine riesige CI-Suite auslösen; für den Sync-Commit wird `[skip ci]` verwendet. Später Tests bewusst per separatem Workflow anstoßen.
- Unabhängige Branches (insbesondere `test/w23a-drawing-update-acceptance-20261028` und PR [#8288](https://github.com/tangency-labs/solidcraft/pull/8288)) **nicht automatisch rebasen, überschreiben oder mergen**.
- Upstream-Zeichnungsarbeit: #9254 ist gemergt, #9821 und #10881 sind noch Drafts. Erst bei Integration Abhängigkeiten kontrollieren und eine gezielte PR-Regression ausführen.
- `drawing.dimension.inspect`-Engine-Tests und SVG-/Paper-Exporttests beweisen **noch kein** Live-U2/UI-Aktualisieren einer bereits geöffneten Zeichnung inklusive gerenderter Maßtexte. Vier vollständige UI-E2E-Fälle bleiben `NOT RUN`.

## Dokumentationsabgleich / Zugriffsstatus

Die maßgeblichen HA-Projektdateien liegen unter `/share/GitHub SolidCraft/` (insb. `UEBERGABE_NEUER_CHAT.md`, `PROJEKTSTATUS.md`, `Chatlauf.md`, `Testberichte/INDEX.md` und die Strategie-/Roadmapdateien). Dieser GitHub-Vermerk ist ab 02.11.2026 die **neue Strategieentscheidung**.

**Achtung:** Der letzte direkte HA-Schreibzugriff wurde vom Zugriffs- bzw. Anmeldedialog abgelehnt. Die HA-Dateien sind dadurch **nicht** aktualisiert; die Nachtragung dort steht noch aus, bis der Benutzer den Zugriff erneut freigibt. Eine Zugriffsablehnung nicht umgehen.
