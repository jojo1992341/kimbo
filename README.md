# Kimbo : sur quelle case me placer ?

Application Windows en Rust (egui/eframe + wgpu).

- Fenêtre toujours au-dessus des autres applications
- Rendu DirectX 12, repli automatique sur OpenGL si DX12 est indisponible

## Compilation (sous Windows)
1. Installer Rust : https://rustup.rs
2. `cargo build --release`
3. L'exécutable est dans `target\release\kimbo.exe`

Tests de la logique : `cargo test`

## Compilation automatique (GitHub Actions)
Le workflow `.github/workflows/build.yml` compile `kimbo.exe` sur un runner Windows.
- À chaque push (ou manuellement via l'onglet **Actions → Run workflow**) : l'exe est disponible dans les *Artifacts* du run.
- En poussant un tag `v1.0.0` : l'exe est aussi attaché à une *Release* GitHub.
