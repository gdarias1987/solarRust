# Application screens

These deterministic fixture captures document the principal Solar Observatory workflows. They
were rendered headlessly at `1600x1000`, scale `1`, with networking disabled; no archive or
desktop state was changed.

| Screen | Purpose |
| --- | --- |
| [Channels](channels.png) | Choose an instrument and inspect its latest capture. |
| [Archive](gallery.png) | Browse locally retained captures. |
| [Capture detail](detail.png) | Inspect one capture before cropping or setting the wallpaper. |
| [Settings](settings.png) | Configure appearance, cadence, retention, language, and crop. |
| [Statistics](statistics.png) | Review archive size, coverage, and recent captures. |
| [About](about.png) | Read the product mission and data-source summary. |
| [Crop editor](crop.png) | Define the desktop crop using the visual editor. |

## Regenerate

Build once, then render each scene with the matching Rune revision:

```powershell
$env:SOLAR_OFFLINE = "1"
cargo build --locked --offline

$scenes = "channels", "gallery", "detail", "settings", "statistics", "about", "crop"
foreach ($scene in $scenes) {
    Start-Process `
        -FilePath ".\target\debug\solar-rust.exe" `
        -ArgumentList "--scene", $scene, "--snap", "docs/screenshots/$scene.png", "--size", "1600x1000", "--scale", "1" `
        -WindowStyle Hidden `
        -Wait
}
```

`Start-Process -Wait` is required on Windows because the application uses the GUI subsystem;
invoking the executable directly from PowerShell does not wait for capture completion.
