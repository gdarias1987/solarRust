# Solar Observatory — Functional Export for Rune

This is a portable product handoff for a **new, cross-platform desktop project using Rune**. It captures Solar Go's existing functionality, not its Go architecture. Copy this document into the new project. Windows, macOS, and Linux are targets; OS-specific wallpaper implementation is intentionally deferred.

## Scope and evidence

Extracted from the working-tree source on 2026-11-09, including existing uncommitted work. This is a source-based inventory, not a runtime certification. No dependency on this repository, Go, HTTP, SQLite, HTML, JavaScript, or Three.js is required by this document.

Preserve user outcomes and domain rules. Rebuild presentation for Rune rather than wrapping the current web dashboard. Do not infer that the backend must remain in Go or that existing user data must be migrated.

## 1. Observation channels

- Browse channel cards with preview image, name, category, ions, and typical temperature.
- Select a channel and inspect its description, wavelength, ions, solar region, and temperature range. Current UI presents temperatures in Celsius and Fahrenheit.
- Persist the selected channel as the default for automatic updates; restore it after restart. Selecting a channel alone does not immediately apply wallpaper.
- Categories: AIA, HMI, and composites. Keep identifiers as strings, including leading zeroes.

### Current catalog

| Category | Channel identifiers |
|---|---|
| AIA | `0094`, `0131`, `0171`, `0193`, `0211`, `0304`, `0335`, `1600`, `1700` |
| HMI | `HMIB`, `HMIBC`, `HMIIC`, `HMIIF`, `HMII`, `HMID` |
| Composite | `COMP211193171`, `COMP304211171`, `COMP094335193`, `COMPHMI171` |

Remote image source: NASA SDO latest JPEG products. Current full-resolution URLs use `https://sdo.gsfc.nasa.gov/assets/img/latest/latest_4096_{product}.jpg`.

Composite product mapping is explicit, not derived from the channel ID:

| Channel | Remote product |
|---|---|
| `COMP211193171` | `211193171` |
| `COMP304211171` | `304211171` |
| `COMP094335193` | `094335193` |
| `COMPHMI171` | `HMImag_171` |

Treat these URLs and descriptions as current application configuration, not a guarantee about future upstream availability. The source catalog contains full descriptions and scientific metadata.

## 2. Acquire and refresh imagery

- Download the latest image for a requested channel and save its metadata locally.
- Check whether a remote image is newer before scheduled downloads, using remote modification time and the last local download.
- A manual Apply action on a channel currently downloads it directly; it is distinct from a conditional update check.
- Run an initial check when the background updater starts, followed by periodic checks.
- Automatically check **the selected channel**, not the whole catalog. Current default interval: five minutes.
- On a newly downloaded scheduled image, attempt wallpaper application with the saved mode and crop.
- Record check outcomes: channel, remote modification value, whether an image was downloaded, and an error when applicable.
- Preserve the distinction between download success and wallpaper-application failure. Current scheduler can retain a successful download even if applying it fails.

Manual channel checking is available in the backend; there is no dedicated check button wired in the current frontend. Preserve it as a capability, not as an existing screen control.

## 3. Local gallery

- Show locally stored images in a grid and an individual-image view.
- Display image identity, channel, date/download metadata, and preview/full image as appropriate.
- Apply a stored image without downloading it again.
- Applying a stored image also makes its channel the selected default for subsequent automatic updates.
- Retrieve images across channels or for a specific channel. Per-channel retrieval exists in the backend; do not assume the current UI has a channel-filter control.
- Current retrieval is limited to 50 records per request. Pagination, search, favorites, and user-driven deletion are not established features.

## 4. Wallpaper behavior

| Mode | Product intent |
|---|---|
| `fit` | Fit the image while preserving its proportions. |
| `crop` | Fill the desktop by cropping as needed. |
| `center` | Center the image. |
| `span` | Request a wallpaper spanning the desktop display area. |
| `solar_crop` | Generate and apply an image from the user's saved crop. |

- Persist mode selection and restore it after restart.
- Changing the saved mode does not itself apply a wallpaper.
- Finish pending mode saves before an Apply action, so the action uses the intended mode.
- Provide current-wallpaper information and, separately, the original image suitable for crop editing.
- Report application failures without reporting false success.
- In Solar Crop mode, a missing crop is an error; do not silently apply an uncropped image.

These are portable intents. Mode support, display targeting, platform fallbacks, and OS mechanisms remain future work, not claims of existing cross-platform parity.

## 5. Solar Crop editor

- Open an editor from settings using the resolved original source image, not a previously generated crop when its original is available.
- Show an empty state if no usable source exists.
- Initialize a centered selection or restore the saved rectangle.
- Support moving the rectangle, resizing with an aspect constraint, and adjusting its scale.
- Use the screen aspect ratio for a newly created selection. Keep selection inside the source bounds and show its dimensions.
- Save coordinates in original-image pixels, together with original width, height, and source reference. Preview/display coordinates must not be stored as source pixels.
- Validate positive dimensions, nonnegative origin, and containment inside the source.
- Reuse the saved crop proportionally when a later source image has different dimensions.
- Save, close/cancel, and clear the saved crop. Closing without saving must not persist editor changes.
- Keep crop existence separate from mode selection: saving a crop does not automatically enable Solar Crop mode.
- Generate a derived JPEG without overwriting the original.

An Earth-overlay data type is still present in backend code, but it is not an exposed current editor feature. Do not promote it to required feature parity.

## 6. Preferences, statistics, and education

### Preferences

- Persist language, selected channel, wallpaper mode, and crop definition.
- Detect preferred system language when no preference has been saved; fall back to English.
- English and Spanish are the actual translated and backend-persistable languages.
- The current selector also lists Portuguese, Chinese, Hindi, Arabic, French, Bengali, Russian, Urdu, Indonesian, and German, but these reuse English text and are rejected by the settings backend. Do **not** advertise them as completed translations in the new project.
- Poll interval, retention count, and storage locations are configurable in the current backend, not editable settings controls in the current UI. Current defaults: five minutes and ten images per channel.

### Statistics

Show download count, stored size in MiB, counts by channel, last update, and last update by channel. Derive definitions from retained download records rather than inventing lifetime totals.

### About SDO

Provide mission context, goals, descriptions of AIA/HMI/EVE, image attribution guidance, and official-source links. Preserve NASA/GSFC/Solar Dynamics Observatory credit. Scientific metadata and mission copy should remain reviewable content, not embedded assumptions about UI technology.

## 7. Interactive observatory presentation

The existing app includes functionality beyond downloading wallpaper:

- Animated solar-system scene with the Sun, eight planets, orbits, moons, Saturn's rings, an asteroid belt, and observatory spacecraft representations.
- Click scene targets to focus the camera.
- Zoom with the mouse wheel or `+` / `-`; `0` refocuses the Sun.
- Middle-button dragging changes camera azimuth.
- A collapsible controls panel exposes Filters, Gallery, Config, and About SDO; Escape closes the crop editor first or collapses the panel.
- Scene interaction does not consume input intended for UI controls.

Record this as existing presentation functionality, not a requirement to reuse Three.js or evidence that Rune can directly run this scene. Native rendering feasibility and visual redesign are separate follow-up decisions. The animated dashboard scene is not an animated desktop wallpaper feature.

## 8. Persistence and file lifecycle

Preserve these behaviors independently of database or filesystem implementation:

- Store source files and metadata consistently, including channel, source URL, remote modification time, download time, size, and local reference.
- Retain a configurable number of images per channel and clean up obsolete files/records.
- Protect the active wallpaper and the saved crop source from retention cleanup; protected files can exceed the nominal retention limit.
- Clean up obsolete generated crops without deleting the currently applied output.
- Defer destructive cleanup when the active wallpaper cannot be determined safely.
- Do not leave a successful-looking image record for an incomplete file write.
- Restore settings and local gallery after restart. Offline remote checks may fail, but existing local imagery must remain usable.

## 9. Acceptance checklist for the new project

- [ ] All 19 channel identities and explicit composite mappings are represented.
- [ ] Selecting a channel persists without applying it immediately.
- [ ] Applying a channel downloads its latest image and updates the selected default on success.
- [ ] Applying a gallery image uses its local source and updates the default channel.
- [ ] Scheduled checks target only the selected channel and skip unchanged imagery.
- [ ] Mode, language, and crop survive restart; mode changes alone do not apply wallpaper.
- [ ] Crop coordinates survive preview scaling and later source-resolution changes.
- [ ] Solar Crop without a saved rectangle produces a clear error.
- [ ] Retention protects the active wallpaper and saved crop source.
- [ ] Download, persistence, and apply errors remain distinguishable.
- [ ] Gallery, statistics, settings, and About SDO remain available.
- [ ] Interactive-scene parity is tracked explicitly rather than silently discarded.

OS adapter implementation and certification are deliberately outside this checklist for now.

## Source map in the original project

| Source | Evidence |
|---|---|
| `internal/sdo/filters.go` | Catalog, descriptions, channel IDs, URLs, metadata. |
| `internal/scheduler/scheduler.go` | Selected-channel polling, update detection, download/apply flow. |
| `cmd/web/main.go` | Application operations, settings validation, gallery limits, apply/crop flows. |
| `web/static/js/main.js` | User workflows, crop interactions, mode-save ordering. |
| `web/static/js/render.js` | Channel details, gallery/settings/statistics rendering. |
| `web/static/js/i18n.js` | English/Spanish text, language detection, placeholder languages. |
| `web/static/solar_system.js` | Animated scene, camera focus, zoom, drag interactions. |
| `internal/wallpaper/service.go` | Mode semantics, derived crop generation, generated-file cleanup. |
| `internal/storage/lifecycle.go` | Protected files, retention, persistence/file maintenance. |
| `internal/config/config.go` | Configurable interval, retention, paths, default channel. |

## Starter instruction for the new project

> Build a new cross-platform desktop solar observatory app using Rune. Treat this document as the functional baseline, not as an instruction to reuse the old implementation. Preserve the observed product workflows and explicitly track unresolved presentation parity. Do not add features or claim placeholder translations are complete. OS-specific wallpaper techniques are deferred: keep their integration boundary separate without choosing mechanisms now. Start by mapping each numbered section to the new project's functional backlog and acceptance criteria.
