# Which desktop shell for a translation editor? Stage 1 results (Windows)

Draft, 27 November 2026. **Provisional results:** Windows only; cross-platform editor consistency (28% of the weight) has not been measured yet.

## Summary

We built the same translation-editor prototype on four desktop shells (Electron, Tauri 2, Avalonia 12, and Flutter with the Fleather editor), sharing a Rust core, and measured latency, memory and size under a protocol registered before measuring and amended along the way (all amendments are public).

- All four stay under the eliminatory typing limit (p95 < 33 ms on the restricted profile) and met every latency target under the standardised conditions.
- Electron, Tauri and Avalonia are in a **technical tie** on the partial score. Flutter trails, mainly because of memory.
- Flutter, with Fleather 1.28.0 as published, showed continuous memory growth over one hour of use. We traced the navigation-related part to the editor library.
- With three fixes to the library, that growth disappears over the one-hour test (sensitivity run, not scored).
- There is no winner at this stage. The heaviest criterion depends on Linux and macOS.

## 1. Question and method

For a desktop editor of structured documents (a CAT tool) with offline work, a sync queue and live preview: which shell should we use?

| Candidate | Version | Segment editor |
| --- | --- | --- |
| Electron | 44.4.5 | ProseMirror |
| Tauri 2 | WebView2 158.0.4370.33 | ProseMirror (same front end as Electron) |
| Avalonia | 12.2.0 (.NET 10) | custom control written for this study |
| Flutter | 3.50.1 | Fleather 1.28.0 |

**Prototype.** DOCX, XLSX and PPTX import; virtualised list; editor only on the active segment, with atomic tags; per-keystroke preview; SQLite with a sync queue; merge and split; translation-memory search. Import, database, search and export live in a Rust core (`catcore`), run as a separate process by three candidates and in-process by Tauri. Keyboard-driven functional tests (T-R2 to T-R7 and T-R9) pass on all four.

**Hardware.** Laptop with Intel Core Ultra 7 255HX, 31 GB, Intel integrated GPU and NVIDIA RTX 5070 Laptop, 240 Hz display, Windows 11, on mains power. **Restricted** profile (primary): 4 cores, 8 GB, integrated GPU. **Full** profile: no limits, NVIDIA.

**Measurement.** Keystrokes sent through the operating system (`SendInput`); each app timestamps the frame following the change. Harness floor: p50 0.50 ms, p95 1.26 ms. Ten repetitions in random order, the first discarded; p50, p95 and a bootstrap 95% interval for p95. A target counts as met only if the upper bound of the interval is below it. Memory: private memory summed over all processes. Scrolling: PresentMon.

**Documents.** Four synthetic ones (120 to 10,000 segments) and three real ones: a US government document with complex layout (329 segments), long running text (*Moby Dick*, an OCR scan with noise, 5,424 segments) and a private PPTX deck (596 slides, 500 MB, 1,718 segments) whose content is not published. Real DGT-TM translation memory, 51,769 EN-GB/PT-PT pairs.

## 2. Results

### 2.1 Latency (p95 in ms; range across documents)

| Metric (target) | Profile | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- | --- |
| M1 key to editor (< 16) | Restricted | 8.6–9.5 | 8.8–10.1 | 6.8–8.6 | 10.9–12.8 |
| M1 | Full | 8.1–8.2 | 8.3–8.5 | 6.2–7.4 | 11.0–11.2 |
| M3 confirm (< 50) | Restricted | 16.5–22.1 | 10.7–12.6 | 15.1–38.4 | 14.9–28.6 |
| M6 merge/split (< 100), worst p95 | Restricted | 34.4 | 23.1 | 32.3 | 33.3 |
| M8 concordance search (< 100) | Restricted | 14.9–23.5 | 11.9–13.4 | 13.3–21.3 | 13.7–24.3 |
| M5 open 10,000 segments (< 2,000), p50 | Restricted | 656 | 483 | 1,117 | 595 |

- All 308 combinations with a target (app, profile, document, metric, step) were met.
- **M2 (key to preview, target < 50)** came out identical to M1: all four apps timestamp the same frame for editor and preview. It adds no information beyond M1.
- **M9 (launch until the app starts opening the document)**, p50 3.4 to 4.2 s with no meaningful difference: every app loads the 51,769-pair translation memory before opening, which dominates. It does not measure bare shell start-up, and cold and cached starts were not separated. One Electron start took 24.4 s, cause unknown.
- **M11 (compiled preview, no target)**: 1.2 to 8.1 s depending on the document, essentially the same for all four, because LibreOffice conversion dominates.
- Merge is refused once per run on real documents (segment pair across blocks), identically on all four.

### 2.2 Scrolling (M4)

Under the registered definition (frames dropped relative to 60 Hz), all four drop 0% on documents long enough for continuous scrolling. The display runs at 240 Hz and the candidates present at different rates (about 240 frames per second for Electron and Tauri, about 100 for Avalonia and 123 for Flutter), a difference this metric does not capture. On the small document the scroll reaches the end, so the value does not measure performance.

### 2.3 Memory

| Private memory, all processes (MB), Restricted | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- |
| 3,600-segment document, just opened | 275 | 264 | 231 | 345 |
| Prolonged use, median of minutes 20–60 | 329 | 330 | 221 | 710 |
| Maximum over 60 minutes | 356 | 354 | 250 | 859 |
| Slope (MB/min) | 0.28 | 0.24 | 0.31 | **7.85** |
| Continuous growth (registered criterion) | no | no | no | **yes** |

Prolonged use is a 60-minute script cycling through typing, confirming, scrolling, merging and splitting, switching between three documents, one open at a time. It was run on the Restricted profile only, once per candidate.

#### Where the Flutter growth comes from

In short tests with one operation type at a time, growth came mainly from switching segments (about 60 KB per switch). Each switch creates an editor and discards the previous one; three places in Fleather 1.28.0 can keep the discarded editor alive: an observer not removed in `dispose`, a frame callback that cannot be removed, and a listener left on an old object.

| Navigation test (about 2.5 min) | MB/min |
| --- | --- |
| Library as published | 18.7 |
| First fix only | 19.1 |
| Second only | 19.9 |
| Third only | 16.6 |
| All three | 0.8 |

Pairs of fixes were not tested. **Sensitivity run (decided after seeing the official result; Amendment 10):**

| 60-minute prolonged use, Restricted | As published (official) | With all three fixes |
| --- | --- | --- |
| Median of minutes 20–60 (MB) | 710 | 382 |
| Maximum (MB) | 859 | 397 |
| Slope (MB/min), 95% CI | 7.85 (7.68 to 8.01) | 0.01 (−0.12 to 0.15) |
| Continuous growth | yes | no |

One run each, same script, documents and seed; in the patched run all 7,200 keystrokes got a response and nothing failed. Functional tests T-R2 to T-R7 and T-R9 pass on the patched package.

Scoring uses the library as published. The patch (`harness/patches/`) has not been reviewed by the Fleather authors; latency was not measured with it, and the functional tests cover neither mouse selection nor the clipboard.

### 2.4 Size

| | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- |
| Installer (MB) | 110.6 | 2.2 | 49.6 | 9.7 |
| Installed folder (MB) | 370.2 | 6.3 | 206.8 | 30.1 |

Same installer script and compression for all four. The Tauri figure **excludes WebView2**, assumed present on the system. No size optimisation was applied (e.g. locale removal or trimming). Update size was not measured.

### 2.5 Effort and ecosystem

An AI agent implemented the study, and the per-requirement hours foreseen in the pre-registration **were not recorded**. Own lines of code (excluding tests and the core): Electron 1,066, Tauri 1,081 (reused front end plus bridge), Avalonia 1,053, Flutter 1,107. Similar line counts do not mean similar effort: Avalonia required writing the tag-aware editor, and Flutter required focus, height and keyboard workarounds in Fleather. See `results/diario-bugs.md` (Portuguese).

Ecosystem (libraries in each candidate's natural language, scored 1 to 5): Electron 4 (no round-trip DOCX library), Tauri 3 (Rust: DOCX unverified and no mature XLIFF; 4 if JavaScript libraries are used in the front end), Avalonia 4 (Open XML SDK and ClosedXML; XLIFF 1.2 by hand), Flutter 3 (DOCX and XLIFF by hand). Sources in `phase0/fichas.csv`, entries 25 to 29; SQLite maturity was not surveyed.

## 3. Partial score

Pre-registered rule; the aggregation choices it left open are in Amendment 11, **written after the results were known** and approved on 27 November.

| Criterion (weight) | Electron | Tauri | Avalonia | Flutter |
| --- | --- | --- | --- | --- |
| Performance (22%) | 4.75 | 4.75 | 4.86 | 4.71 |
| Memory (17%) | 4 | 4 | 5 | 1 |
| Ecosystem (11%) | 4 | 3 | 4 | 3 |
| Installer (5%) | 1 | 5 | 1 | 1 |
| **Partial, 55 weight points** | **4.03** | **4.19** | **4.38** | **2.89** |
| Variant with effort by line count (all 5), 72 points | 4.26 | 4.38 | 4.53 | 3.38 |

**Sensitivity** (`results/fase4/sensibilidade.csv`), over 60 combinations of calculation choices:
- Tauri ranks first in 36 and Avalonia in 24. The order between them **depends on the calculation choice**.
- Tauri and Avalonia are within 10% of the leader in all 60; Electron in 45; Flutter in none.
- Using memory with a freshly opened document instead of prolonged use puts Tauri ahead (4.50 vs 4.38) and raises Flutter to 3.50.
- With the patched Flutter memory figures, its memory score rises from 1 to 4 and its partial score from 2.89 to 3.81, still 13% below the leader. This variant is not part of the 60 combinations.
- Consistency (28%) can reverse the order: on the main partial score, a consistency advantage above 0.37 points would put Tauri ahead of Avalonia, and above 0.69 would do so for Electron.

## 4. Eliminatory criteria at this stage

| Criterion | Status |
| --- | --- |
| Stable on all three systems | not assessed (Stage 2) |
| Protected tags behave the same on all three | not assessed (Stage 2) |
| Typing p95 < 33 ms on Restricted | all four pass |
| Commercial licence with no per-user fee | the four frameworks and the editors used are permissive open source; Avalonia's paid editor was not used |

## 5. Threats to validity

- **One computer** and one operating system.
- **Static screen (Amendments 8 and 9).** With nothing else being redrawn on screen, typing and confirm p95 rise by about 6 to 9 ms on Electron, Tauri and Avalonia; Flutter barely changes. Targets are judged with a small animation on screen. In four complete static-screen rounds, 107 of 112 combinations meet the target, 4 are borderline (Avalonia) and 1 misses (Tauri, long real text, M1: p95 17.1 ms). The mechanism is unknown, as is what was redrawing the screen during the seven retained rounds run before the animation was added.
- The invalidation rule uses metric M1 itself and was created after a first look at the results. Discarded rounds are reported. The Full-profile scrolling rounds have no counterpart to check the effect.
- **Mid-course corrections:** focus guard in the driver and retry of failures (Amendment 7; earlier rounds lacked it); fixed pairing between keystroke and response, with everything reprocessed; Avalonia switched to a DXGI swap chain so scrolling could be measured (Amendment 6).
- **In-app frame timestamps** exclude display scan-out, and the method differs per framework.
- Electron and Tauri **share a front end**, so Tauri's figures do not represent the effort of starting with Tauri. Both received the same ProseMirror fix for arrow keys next to tags.
- **Avalonia** reflects an editor written for the study, not the vendor's RichTextEditor. **Flutter** represents one editor (Fleather), not the framework as a whole.
- **Memory:** one 60-minute run per candidate, Restricted only, one document open at a time (the pre-registration mentioned three). Flutter's run was repeated because of an unresponsive gap in the first one.
- **M8** uses 51,769 pairs, not the planned 100,000, and only concordance mode (substring match): the translation-memory similarity mode required by Amendment 3 was not measured. CPU temperature unavailable. No XLSX export. Compiled preview on demand only. CJK IME out of scope.
- **AI use:** implementation, harness, analysis and this draft were produced with an AI agent, under the author's decisions.

## 6. Next steps

Stage 2 (Linux and macOS) measures stability, atomic tags and editor consistency, currently the largest unknown. It starts only with explicit authorisation.

## Reproduction

`PREREGISTRO.md` (protocol and amendments, Portuguese), `spec/` (specification and scripts), `harness/` (driver, rounds, analysis: `resultados.py`, `pontuacao.py`), `results/` (raw and consolidated data). Private and real documents are not versioned; synthetic ones are generated by `corpus/gen_corpus.py`.
