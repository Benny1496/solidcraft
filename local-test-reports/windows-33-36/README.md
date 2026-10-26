# Windows-Regressionsstand Tests #33–36 (WIP)

**Nicht als vollständig validierter Upstream-Beitrag zu verstehen.**

Basis: `79f2a63870c7b8db29913b96464b92a6055bbc37` auf `feature/browser-automation`.
Gesichert auf Windows am 26. Oktober 2026 (Test #36).
18 geänderte getrackte Dateien, 73 Einfügungen und 27 Löschungen.

- Test #33: 4/4 gezielte Nachtests bestanden.
- Test #34: 11/11 gezielte Nachtests bestanden.
- Test #35: 12/24 gezielte Nachtests bestanden.
- Summe: 27/39 gezielte Nachtests bestanden, **12 Fehler weiterhin offen**.
- Offene Gruppen: 4 Undo/Zustand, 4 Robustness-Goldens, 3 Kernel-Topologie, 1 Drawing-UI-Fenster.

Die bestehenden `next_id`-Anpassungen ändern vor allem **Testvergleiche**:
Beim Undo darf der ID-Zähler absichtlich nicht sinken. Daraus folgt **nicht**,
dass die 12 verbliebenen Produktprobleme bereits gelöst sind.

Der Code dieses Branches ist eine bytegenaue Sicherung des Windows-Teststands,
noch **keine vollständige Rust-Regressionsfreigabe**. Vor einer Übernahme in
andere Entwicklungsbranches sind Review, konkrete Regressionstests und der
Abgleich mit aktuellem Upstream erforderlich.

Der parallel laufende ESP32-GitHub-Workflow `#2` checkt weiterhin den alten
Branch `feature/browser-automation` aus und **testet diese Sicherung nicht**.

Git-Diff-SHA256: `755962c514b945c91db40e1f646acdfa1a86a2646149ff0411a274079808a701`.
Der beiliegende Patch dokumentiert die 18 Quelländerungen als zusätzliches Archiv.
