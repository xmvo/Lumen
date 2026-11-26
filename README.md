# Lumen

Lumen runs an Xbox Series build of a game you own on Windows through Direct3D 12. The game's own
x64 code runs natively; its console GPU code is translated for a PC graphics card.

**This repository holds no game files, code or assets.** You need your own copy of the game.
Lumen is not affiliated with or endorsed by the game's developer or publisher.

Last updated: 2026-11-26. What we measured ourselves is said to be measured; the rest is marked.

## Which build is for you

| Your graphics card | Use | Where |
|---|---|---|
| **NVIDIA** | the NVIDIA build: a ready zip with a launcher, no build tools needed | branch [`nvidia`](../../tree/nvidia), [releases](../../releases) |
| **AMD Radeon** | this branch (`main`), built from source | below |

The two are different code lines. The NVIDIA build is the one we develop and test; its README has
its measured status, known problems and settings. Do not use the NVIDIA zip on a Radeon card or
this branch on an NVIDIA card:

- **This branch on an NVIDIA card** (measured, RTX 3060 12 GB): 2.0 to 2.7 fps, and 8 of the
  game's 64-lane compute pipelines are refused. Its optional speed switches lost the device there.
- **The NVIDIA build on a Radeon card**: not tested.

## This branch: the AMD build

This branch is [DexNom Runtime](https://github.com/DexNom/VIctory) at its commit `2e01ab4`, a
fork of Marvisak's [VIctory](https://github.com/Marvisak/VIctory) tested by its authors on a
Radeon RX 7900 XTX, with two changes of ours:

- **Shade is lit by default.** With the ray-structure builder off, the game's indirect light
  resolves to nearly nothing and whatever the sun does not reach is black (faces, clothes, rooms).
  The launcher starts the runtime with `VICTORY_AMBIENT=0.6` unless you set it yourself or ask for
  ray tracing (`--rt`). The stand-in is fxceless/victory-fork's pull request #917, after
  [Belobug's fork](https://github.com/Belobug/VIctory-Belobug-AMD).
- `VICTORY_RELAY_QUEUE=1` (off by default): the landing thread's fetch runs on a queue of its
  own. It gave no gain on our PC.

**We have never run this branch on a Radeon card.** We have no AMD hardware. What it does on
Radeon is what DexNom's own notes below say, as of that commit; we cannot confirm them. There is
no ready zip for this branch: follow the build steps in those notes.

## Licence and credits

GPL-3.0-or-later, as VIctory is. The base is the work of Marvisak and VIctory's contributors, of
DexNom and the runtimes it builds on, and of the authors named in the notes below; their history
is kept in this repository. Our own changes were written with
[Claude Code](https://claude.com/claude-code).

---

# DexNom Runtime's own notes

**What follows is DexNom Runtime's README as it was at the commit this branch is built on. Its
results and its test machine (a Radeon RX 7900 XTX) are DexNom's, not ours.** Where it says
"DexNom Runtime", the same steps apply to this branch.

## Introduction

DexNom Runtime is a community fork of Marvisak's VIctory for Windows x64. It runs an Xbox Series executable from your own game installation and translates its GPU work for Direct3D 12. It builds on the Radeon runtime tested at `e061c69032b84ec6e2ec17340469cd689500dafd` and adds fixes for static sampler selection, wave64 lane masks and integer render targets. Game files, executables, saves and generated shader caches stay on your own computer.

## Run it on your PC

1. Use 64-bit Windows and a current Direct3D 12 driver. The game uses mesh shaders and Shader Model 6.6. The Radeon gameplay results and known issues are listed below.
2. Install Rust for Windows and Visual Studio Build Tools. In the Visual Studio installer, select **Desktop development with C++**, including the x64 MSVC toolset and a Windows 10 or 11 SDK. [Rust installer](https://rustup.rs/) · [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
3. Download this repository using **Code → Download ZIP**, extract the ZIP to a writable folder, and double-click **Setup.cmd**. Select the Series X game executable inside your own extracted game installation. Keep its accompanying files together. Setup downloads the pinned build dependencies, verifies the BCPack format tables, and compiles the three Windows programs locally. [Download source ZIP](https://github.com/DexNom/VIctory/archive/refs/heads/main.zip)
4. Open **DexNom Runtime** from the Windows Start menu. Use a controller for the game's first-time setup. The launcher and your local data are in `%LOCALAPPDATA%\DexNom Runtime`. Saves are in `state`, shader caches in `cache`, and per-run diagnostics in `logs`. The file `logs\latest.txt` points to the latest game log.

To update, close the runtime, extract the new source ZIP, and run **Setup.cmd** again. It reuses the saved game path and the same save directory. To select a different installation, pass its executable path to Setup from PowerShell:

```powershell
.\Setup.cmd -GamePath "D:\My Games\Series X\game_scarlett_master.exe"
```

To warm the shader cache, open `victory-warm.exe` in the installed runtime directory. The log lists translated and refused shaders. Graphics pipelines also depend on draw state encountered during play. If you see a missing `VCRUNTIME140.dll` error, install Microsoft's x64 redistributable. [Microsoft Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)

For a manual build, run `.\Initialize-FormatTables.ps1`, then `cargo build --locked --release -p victory --features diagnostics --bins`. The release binaries are in `target\release`. Place all three together outside the checkout and pass the game executable path to `victory-play.exe`. Custom data and cache directories are documented in the configuration example. [Configuration example](victory.example.toml)

## Try experimental ray tracing on Radeon

After running **Setup.cmd**, open **DexNom Runtime (Experimental RT)** from the Windows Start menu. This starts the game with the acceleration-structure builder, ray traversal and lighting resolve enabled. The shortcut uses your existing saves. Open **DexNom Runtime** for the usual renderer.

To start the same RT configuration from PowerShell:

```powershell
& "$env:LOCALAPPDATA\DexNom Runtime\victory-play.exe" --rt
```

Chapter 1 City produced in-range ray and triangle intersections on an RX 7900 XTX. The game builds and traverses its acceleration structures through translated Direct3D 12 compute shaders. Frame times can still reach several seconds. Interior lighting, shadows and performance vary by scene. [Ray-tracing implementation](docs/fork-ray-tracing.md) · [City testing](BENCHMARK.md)

## Changes in this snapshot

- Lumen: with the ray-structure builder off, the game's indirect light resolves to nearly nothing and whatever the sun does not reach is black (faces, clothes, rooms). The launcher now starts the runtime with `VICTORY_AMBIENT=0.6`, which replaces that image by one constant; the game's own lighting applies it with the surfaces' colors. Set `VICTORY_AMBIENT=off` to see the game's own output, or another level; `--rt` leaves it off. From Belobug's VIctory-Belobug-AMD by way of fxceless/victory-fork pull request #917. Checked on an RTX 3060 only so far (the first cell: walls, clothes and posters lit); frame rate unchanged.

- Pixel shaders that read neighboring pixels in a 2×2 group now preserve the values at triangle edges. On an RX 7900 XTX, this reduced incorrect edge pixels from 32 to zero in each of two adjoining-triangle tests. Cube-map sampling also selects the correct mip level for explicit gradients and honors the shader's minimum mip level.
- The ray-tracing memory directory can grow beyond 1,023 backing buffers. Each recorded draw or dispatch uses only the views available when it was recorded. Older command lists keep their original views.
- Streamed textures can upload when only their middle pages are resident. The runtime merges overlapping memory notifications and rescans tracked textures if the notification queue fills. Previously used depth-only pipelines now warm ahead of speculative material pipelines to reduce missing shadows during loading.
- Some dock objects still appear black in the City save. The same distant dock fixtures appeared dark with the ray-structure builder disabled. The effect of these changes on those objects and on gameplay FPS is still being tested.
- GPU readback identifies changed pages and combines adjacent copy ranges. This targets CPU/GPU synchronization and transfer overhead.
- Ray-structure builder dispatches now honor the game's thread counts and mask excess lanes in the final group. Read-only traversal no longer declares the entire guest memory directory writable when the shader sends its output to another buffer or texture.
- Texture handling searches for contiguous descriptor slots and admits already-resident textures under memory pressure. Earlier changes address publication order, placed-texture ownership and supported render-XOR layouts.
- Shaders can choose between two static samplers across a uniform branch. This recovered 40 shader variants from the tested archive. Another fix recovered one voxel visibility shader by recognizing a saved and restored wave64 lane mask. Mixed render targets retain their individual write masks when an integer target bypasses blending.
- The runtime preserves the game's vertex output interpolation and fixes output signature ordering. All 2,191 archive vertex shaders passed the Microsoft DXIL validator, including 18 water shaders. New textures and retries share the upload budget, and a previously refused buffer view is checked again when its backing allocation changes.
- The runtime resolves guest addresses from ray-tracing descriptors, preserves raw buffer offsets and prepares the memory each command list reads. Texture uploads can proceed while Windows remaps unrelated memory. In-game ray-traced lighting is still experimental.
- Five additional shadow compute shaders compile and create pipelines on the RX 7900 XTX. The translated ray-node regression also exercises three reflections through the guest-page directory. Changes confined to the standalone execution tests now preserve cached shader translations.
- Shader translation preserves the guest wave width for lane permutations and handles heap-indexed texture-size queries. Save handling flushes copied blobs before publication and can recover a complete checkpoint.

The source ZIP builds with the Windows setup script. The test report covers shader validation, a Radeon pipeline check, gameplay frame times and the remaining interior lighting issue. [Test and benchmark report](BENCHMARK.md)

### Testing CPU submission workers

For a local experiment, open PowerShell and run the following after setup. The process uses four CPU workers for eligible upload batches and can record independent transition command lists on the shared worker pool. The final GPU queue, fence signals and resource transitions retain the game's submission order. Compare the same save and camera after the initial streaming period. The default launcher uses the baseline settings.

```powershell
$env:VICTORY_FILL_WORKERS = '4'
$env:VICTORY_PARALLEL_PRELUDES = '1'
& "$env:LOCALAPPDATA\DexNom Runtime\victory-play.exe"
```

## Credits and licensing

- Marvisak and the VIctory contributors created the original runtime. [Marvisak/VIctory](https://github.com/Marvisak/VIctory)
- ZSlayerHQ and the contributors to the victory-world line provided the rendering baseline used here, including the Radeon work from triippywhale and asdaasdsaddf. [victory-world-runtime](https://github.com/ZSlayerHQ/victory-world-runtime) · [Integrated revision](https://github.com/ZSlayerHQ/victory-world-runtime/commit/99f8834aa16b98d7f3943b8de262a8d4857bda7b)
- The XCL contributors published an independent Xbox Series compatibility layer. Their work informed our investigation of shader translation, CPU/GPU memory synchronization and performance. [xcl-dev/xcl](https://github.com/xcl-dev/xcl)

The upstream README preserves the original credits and historical test results. [Upstream README](README.upstream.md)

VIctory source is licensed under GPL-3.0-or-later. The upstream project identifies the required BCPack format tables as separately supplied data outside that license. This source archive omits that file. The setup script verifies a local or upstream copy before use. [License](LICENSE) · [Upstream asset policy and table notice](docs/asset-policy.md)
