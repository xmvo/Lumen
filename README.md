# Lumen, NVIDIA build

Lumen runs an Xbox Series build of a game you own on Windows through Direct3D 12. The game's own
x64 code runs natively; its console GPU code is translated for a PC graphics card. This branch
(`nvidia`) is the build for NVIDIA cards.

**This repository holds no game files, code or assets.** You need your own copy of the game.
Lumen is not affiliated with or endorsed by the game's developer or publisher.

Everything in this section was measured or checked by us and is dated. Where something was not
measured, it says so. Last updated: 2026-11-26.

## Honest status

**It runs and you can play, slowly.** It is not a finished port.

| | State | How we know |
|---|---|---|
| Boot, menus, loading screens | work, 15-30 fps | every run |
| Opening cutscene and first gameplay (the cell, the yard) | work | every run that reaches the world |
| Speed in gameplay | **about 5.5 to 5.9 fps** | timed runs on the test PC below, builds up to release 0.4 |
| Time from start to gameplay | about 4 to 5 minutes with warm caches | timed runs |
| Further into the story | not tested by us | |
| Ray tracing | experimental, did not hold on the test PC | see below |

The frame rate is limited by the processor, not the graphics card: the card sits idle for most of
each frame while one thread of the layer prepares and submits the game's draws (about 3,500 a
frame) and keeps the card's copies of game memory up to date.

### The test PC

Every number in this section is from this one machine:

- Intel Core i7-4790 (4 cores, 8 threads)
- NVIDIA GeForce RTX 3060, 12 GB
- 16 GB of RAM, page file 32 to 48 GB
- Windows 11
- Series X profile (the full-resolution one)

It is a weak processor and little memory for this job. A faster processor should do better; we
have no measurements from other machines. One outside run on an RTX 5090 reached the world and
drew it; no frame rate was reported back.

## Download and run (no build tools needed)

Releases are on the [releases page](../../releases). The newest is **0.5**
(`Lumen-NVIDIA-0.5.zip`).

1. Unzip the whole `Lumen` folder somewhere outside Program Files.
2. Run `LumenLauncher.exe`. The files are not signed, so Windows SmartScreen asks once.
3. Settings, Change: pick `game_scarlett_master.exe` in your own copy of the game.
4. Close your browser and other large programs.
5. Press Play.

The first Play prepares every shader of the game from the `shaders.rpf` beside the game file you
picked (25,889 found in ours: 25,689 translate, 200 do not). That takes a few minutes once per
build. After that Play starts the game directly.

What is in the zip: `LumenLauncher.exe` (source in `launcher/`), `bin\lumen.exe` (this branch's
layer), `bin\lumen-rt.exe` (the ray tracing build, branch `rays-pr905`), `README.txt`,
`CONTROLS.txt`.

### What you need

- Windows 11 (Windows 10 needs the WebView2 runtime for the launcher)
- An NVIDIA card with 12 GB or more (only the RTX 3060 12 GB was tested by us)
- 16 GB of RAM at the very least; with 16 GB it runs out of memory in some runs (see Known
  problems). 32 GB or more is far better.
- Your own unpacked copy of the game

## Launcher settings

| Setting | Default | What it does | Checked? |
|---|---|---|---|
| Lit faces | on | Fills in the indirect light the game expects from ray tracing, so shade is not black (`VICTORY_AMBIENT`) | yes, in runs |
| Light level | 0.6 | Strength of that fill light | 0.6 looked balanced in the first rooms |
| Skin light fix | on | Also fills five more light images the game's resolve writes, which skin and outlines read | seen in one run: the cell scene is clean with it; not yet confirmed on a face in motion |
| Fast mode | on | Shows frames behind the queue (`VICTORY_ASYNC_PRESENT`); about one frame a second more, uses more memory | yes |
| No pop-in | on | A frame waits up to 4 s for a pipeline still being built instead of drawing without it (`VICTORY_PIPELINE_WAIT_MS=4000`) | not play-tested |
| Full textures at once | off | Uploads textures whole instead of small levels first; for 32 GB of RAM or more | not tested |
| Ray tracing (experimental) | off | Runs `bin\lumen-rt.exe` instead | see below |
| Rays at full resolution | off | Series X profile with rays; its authors ran out of memory with 32 GB | not tested |

Helper threads are chosen from the processor: half its logical cores, 4 at least, 16 at most.

Closing the launcher never closes a running game. Logs are in the `logs` folder beside it.

## Known problems

- **Out of memory while the world loads** on 16 GB. The game commits 21 to 24 GB. A larger fixed
  page file (32 GB to start) and closing other programs help; turning Fast mode off uses less.
- **A crash inside the NVIDIA driver** (`nvwgf2umx.dll`, access violation) while the world loads,
  about one run in seven here. Starting again works. Not fixed.
- **Skin can turn black, speckled or purple** while characters or the camera move, and blocks can
  show along outlines. Release 0.5 carries a fix aimed at this; it is not yet confirmed on faces.
- **Late textures and low-detail models** at the start of cutscenes: the game streams them while it
  plays and they arrive later than on the console.
- **The first pass through a place hitches**: the graphics driver builds each pipeline the first
  time the game uses it, and keeps it. This cannot be done ahead; which pipelines exist is only
  known when the game asks for them.
- **200 shaders do not translate**: 195 are the game's voxelizer (a console geometry stage the
  translator does not support at all) and 5 are shadow ray tracers. In a 25-minute session the
  game bound 2,871 different shaders and none of these, so they cost nothing in what we played.
- Sound, saves and controller input: not tested by us beyond ordinary keyboard-and-mouse play.

## Ray tracing (experimental)

The switch runs a second layer, `bin\lumen-rt.exe`: the live ray tracing of pull requests #814
(by staronthehorizon) and #905 of fxceless/victory-fork, kept on branch `rays-pr905` unchanged.
It has none of this branch's fixes or speed work.

- Its authors ran it on an RTX 3080 with 32 GB of RAM: interiors lit by bounce light, grainy,
  300 to 440 ms a frame, on the Series S profile.
- **On our test PC it did not hold**: the ray structure builder started (all 53 of its programs
  had pipelines) and about 20 seconds later the graphics device was lost. The PC had under 400 MB
  of memory free at that moment, so memory may be the cause; we do not know.
- We have not run it on any other card.

## What is in this build

The base is [fxceless/victory-fork](https://github.com/fxceless/victory-fork) (by hyphennn and
contributors), itself a fork of [VIctory](https://github.com/Marvisak/VIctory) by Marvisak. On top
of its `main` at `0b0074e`:

- its pull request #770 (a bundle of 22 pull requests, put together by ExoCalibra) and pull
  requests #929, #916, #919, #947, #927, #907, #934, #913 and #932;
- the paired form for the console's 64-lane compute programs on 32-lane cards (its pull request
  #854): without it NVIDIA cards refuse many of the game's lighting programs;
- its pull request #834 (compute dispatches past 65,535 groups), used for the one program that
  needs it (the lighting pass): applied to all programs it gave wrong colours on our card, for a
  reason we did not find;
- the stand-in for indirect light (its pull request #917, after
  [Belobug's fork](https://github.com/Belobug/VIctory-Belobug-AMD)), extended here to the
  resolve's other light images;
- two shader fixes after [DexNom Runtime](https://github.com/DexNom/VIctory): pixel programs read
  their 2x2 group through quad operations, and a cube map's level from gradients is not scaled
  twice;
- our own changes: locks that readers no longer queue for (buffer lookups, descriptor slots, the
  write-watch journal and guards), retired objects in sixteen lists, a lock around tile mapping
  while an upload records, a retry when an upload buffer is refused for want of memory, a hook for
  the title's watchdog, a wait for the decoration bake's textures;
- two opt-in switches that are **off** because they did not help or were not measured on the
  test PC: `VICTORY_HEAP_WORKER=1` (upstream's texture thread, 3.6 fps here against 5.5 to 5.9)
  and `VICTORY_WATCH_RESERVATION=1` (after Kelvaroq's patch 0045, not yet measured);
- the launcher (`launcher/`, Tauri).

Tried and not taken, with the result on the test PC: [Kelvaroq](https://github.com/fablinoo/kelvaroq)
as a whole (1 to 2.7 fps in the world, one lost device); a full merge of the fork's newer `main`
(the two lines rebuilt the same memory tracking differently, and its `main` lacks the 64-lane
form above).

## Build from source

Rust (the toolchain `rust-toolchain.toml` names) with the MSVC tools and a Windows SDK.

```
cargo build --profile diag -p victory --features diagnostics
```

The layer is `target\diag\victory.exe` (the releases ship it as `bin\lumen.exe`). The launcher:

```
cd launcher\src-tauri
cargo build --release
```

Nothing from the game may be committed: `cargo xtask policy` runs before every commit (see
`docs/asset-policy.md`).

## Branches of this repository

| Branch | What it is |
|---|---|
| `nvidia` | this build |
| `main` | the build for AMD Radeon, on DexNom Runtime |
| `rays-pr905` | the ray tracing build, upstream's pull request #905 as it is |

## Credits and licence

GPL-3.0-or-later, as VIctory is. All of the base is the work of Marvisak, of hyphennn and the
contributors of fxceless/victory-fork, and of the authors of the pull requests and forks named
above; their history is kept in this repository. [XCL](https://github.com/ivall/xcl) (GPL-2.0-only)
was a reference for the fork this builds on. This build's own changes were written with
[Claude Code](https://claude.com/claude-code).

---

# Notes inherited from the fork this build is based on

**What follows is the README of fxceless/victory-fork as it was at our base, kept for its
description of the switches and the internals. Its status, its numbers and its test machine (an
RTX 4070 with an i7-14700KF) are that fork's, not this build's, and parts of it are out of date
here.**

## The fork's own introduction

A work-in-progress fork of [VIctory](https://github.com/Marvisak/VIctory), a
compatibility layer that runs an Xbox Series title natively on Windows from
files the user supplies. This fork follows upstream
`main` and adds fixes aimed at one goal: getting the game's world to render
correctly and playably on one particular PC.

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's
developer or publisher.

## Credits

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors is the base of everything here: the loader, the runtime
  services, the Direct3D 12 back end and the shader recompiler. Licensed
  GPL-3.0-or-later; so is this fork. Upstream's history is kept intact below
  the single commit that carries this fork's changes.
- **[XCL](https://github.com/ivall/xcl)** by ivall and its contributors
  (GPL-2.0-only) was used as a reference for how the same title behaves under
  another layer. Fixes inspired by it were written anew in VIctory's code.
- The guard over every view of a memory page is code by **myokex**
  ([victory-amd](https://github.com/myokex/victory-amd), commit `c6dc4d8`), which
  adapts pull request #115 here by Spexivus, its correction by joroyt and a part of
  VIctory #1122 by tightfall. The read-beside fix for kernel reads into title
  memory is by **Joshua Odom** (commit `10ea38e` of the same fork family). Both
  are carried here unchanged apart from one merge conflict.
- This fork's changes were developed with [Claude Code](https://claude.com/claude-code).

## Status of this fork (2026-11-25)

**Not playable yet: 2.5 to 4 frames per second, and shadowed areas are black.**
Tested on one machine: Windows 11, Intel i7-14700KF, NVIDIA RTX 4070 12 GB,
32 GB RAM. The Series X profile is used throughout.

| Stage | State in this fork |
|-------|--------------------|
| Boot, menus, start screen | works, 30 fps (as upstream) |
| Opening cutscene | plays from start to finish, in order, lit: title cards, credits cards, night and dusk shots |
| Saves | the game autosaves a checkpoint and loads it again |
| First gameplay scene (the prologue) | renders: characters, textures, sky, sunlit exterior; HUD, pause menu and pad input work |
| Light in shadow | missing: the game's ray-traced bounce light and ray-traced lamp and sun shadows do not run yet, so anything the sun does not hit directly is black |
| Frame rate in the world | 2-4 fps: 420-450 ms per frame by default, 240-350 ms with the newer switches (`VICTORY_RELAY=2`, or `VICTORY_LATE_WAIT=3 VICTORY_WIDE_BACK=1 VICTORY_CARD_BUFFERS_MIN_MB=1 VICTORY_CARD_BUFFERS_MB=1024`); it started at 3-6 seconds per frame |
| Stability | no driver hang or device loss in normal runs (a hang introduced by the wave64 update was found and fixed: its barrier form for 64-lane programs is now off unless `VICTORY_WAVE_WIDE=1`) |

### Newest (2026-11-25, later the same day)

**Update, the same night: the first room now renders right on the test
machine** (walls, textures, people, lamps), at 190 to 450 ms a frame (2 to 5
frames a second): the picture work has caught up, the frame rate has not.
What did it, on top of the list below:
- Descriptor records the title copies into its heap as plain bytes are
  mirrored before every submission (pull request #305 by Flyander, after
  upstream VIctory 86a5ed6). Measured here: about 40,000 records a frame
  that no call had announced; their host slots were empty or stale.
- A buffer view that does not start on an element of its buffer's host
  copy gets a copy of its own bytes instead of being refused (after
  upstream 86a5ed6): 3 to 6 such views a frame.
- Room on the card is kept for the render targets and buffer copies a
  world makes in its first frame: 98 buffer copies are made (945 MB) where
  the runs before made 1, and the processor no longer compares 550 to
  710 MB a frame looking for shader writes (25 MB now).
- Levels on demand ran on the card for the first time
  (`VICTORY_LEVELS_ON_DEMAND=1`, still off by default; with the fixes of
  pull request #747 by AscilCH): 3.1 GB of the card not allocated for
  texture levels the title does not hold, no device loss.
- Guarded blocks under storage writes are opened by default; 14 more
  shader programs translate; a program is translated with first-half
  interpolation pairs only if it was refused for one (with that on for
  every program, 8,551 working pixel programs came out changed).
- Frame rate, first cuts on the processor side: the descriptor sync uses a
  cheap fingerprint and leaves draw-table slots to their draws (the pass
  over the heap went from 70-220 ms a frame to about 3), and memory the
  guards already hold is not asked of the write watch (pull request #670
  by Jonirulah). Card copies' pieces can come down inside their own
  flights (pull request #648 by Jonirulah, `VICTORY_CARD_FETCH_INFLIGHT=1`,
  off by default: the copy-back thread's wait went from 181 ms a frame to
  0 here, but the card got busier, so no gain is claimed yet). The frame is
  still 250 to 380 ms: the card is the limit now, most of it the
  per-record clamp before each of some 6,700 indirect draws a frame.
- Pull request #220 (cheaper indirect draws) is IN this tree now, its
  windows off by default (`VICTORY_INDIRECT_WINDOW=1`). History, so nobody
  repeats it: with windows on it lost the device three times here. Its own
  checker (`VICTORY_INDIRECT_WINDOW_CHECK=1`) then showed one list drawing
  90 records of 2^24 vertices from a window, where the per-draw path lets
  eight through; a record of 2^23 vertices an instance or more now runs
  empty. Since that fix: eight runs with windows on, seven without a
  device loss and one that lost the device 44 s into the world (cause not
  found; see the driver watch below). One earlier loss after the fix had
  the in-flight fetch (#648) on as well; do not combine the two. The
  in-flight fetch itself measured slower here in two runs and stays off.
- Frame rate, second night (an RTX 4070; the first room went from about
  3.5-4 to 6-7 frames a second with ALL of the switches below; the
  defaults are unchanged and give about 4.5-5):
  - `VICTORY_ASYNC_SUBMIT=1`: submissions are prepared and sent by a
    thread of the layer, not by the title's render thread, which a
    profile showed inside the submit path for 257 of 292 ms a frame.
  - `VICTORY_QUEUE2=1`: fetches of what shaders wrote go to a copy queue
    (brought back, with the fix for a piece fetched beside a later
    flight); the copy-back thread's wait fell from about 125 to 7 ms a
    frame.
  - `VICTORY_INDIRECT_WINDOW=1`: see above.
  - `VICTORY_TEXTURE_NO_WAIT=1`: a texture refresh no longer waits while
    the title remaps memory; the loading phase had 5 readings over a
    second where it had 18.
  - `VICTORY_MERGE_GAP=64 VICTORY_MERGE_RUNS=4096`: title bytes put in
    under flights' shader writes are joined over small gaps (NOT exact:
    read the comment at `merge_gap` before using it).
  - On by default: fills of upload memory and read-only buffer copies no
    longer wait for flights and are checked against what the flights then
    wrote (`VICTORY_FILL_WAIT=1` waits as before; no mismatch in any run
    so far); up to 512 idle host lists and texture staging buffers are
    reused instead of made and freed every frame.
  - `VICTORY_PROFILE=1` (diagnostics builds): a sampling profiler inside
    the layer; it is how the above was found.
- Frame rate, third session: measuring first (same RTX 4070, first room).
  - A frame log (`VICTORY_FRAME_LOG`, on in diagnostics builds): every
    frame's length, and for the usual frame and each long one what it was
    made of (the render thread's time in the present call, the submit
    thread's work, copies, copy-back, recording, texture work, faults,
    disk). A card meter (`VICTORY_CARD_METER`) adds the card's time by
    kind of list from a timestamp at each list's start and end.
  - What they showed, windows off: a usual frame of 136 ms; the title's
    render thread waits 77 ms of it in the present call for the submit
    thread, which works 118 ms (copies of title memory 41, copy-back 40);
    the card is 85-98 % busy, of about 130 ms a frame 16 ms in our lists
    that bring card copies up to date, 10 ms in our compare lists and 102
    ms in the title's own lists with the per-record clamp. Frame time is
    linear in draws (about 32 microseconds a draw all-in): the title draws
    about twice as much on some frames, which are the long ones.
  - The regular dips of earlier test runs were the test harness: a saved
    frame (`VICTORY_CAPTURE_WORLD`, every 8th world frame) is read back
    and written as a PNG inside the present, 235 ms each.
    `VICTORY_CAPTURE_WORLD_EVERY=64` for runs that measure speed.
  - `VICTORY_INDIRECT_WINDOW=1` measured again with those instruments. A
    CORRECTION: the first figures given here (6.0 against 8.9 frames a
    second) compared two runs that stood in different scenes during the
    measured window (about 4,300 against 1,550 draws a frame) and are
    withdrawn. Compared at equal draws, by a line fitted to each run's
    frame times against its draws per frame: windows off 145 ms at 4,500
    draws and 271 ms at 10,000; windows on with `VICTORY_COPY_PLAIN=1`
    124 ms and 211 ms. On screen: 7-8 frames a second in the first rooms,
    10-12 in their lighter views.
  - A driver watch (on by default; `VICTORY_DRIVER_WATCH=0` off): the
    layer subscribes to the system's event log and logs every entry of a
    display driver as it is recorded; the first one turns draw windows
    off for the rest of the run. Every device loss here had such entries
    2.7 to 55 seconds before the device was removed, and no run without
    them lost its device.
  - A copy between two title textures that share one host texture is no
    longer recorded (the host does not allow a texture copied onto
    itself): 402 of them in a five-minute run.
  - The list of shader-visible heaps is a reader-writer lock (recording
    threads waited 48 ms a frame, summed, behind the heap sync); the heap
    sync looks at a changed page's records under one lock, fingerprint
    first; guarded view memory is left out of a submission's write-watch
    askings ahead (after triippywhale).
  - `VICTORY_COPY_PLAIN=1`: copies of title memory made in place by this
    process, with a handler that falls back to the kernel's copy on a
    missing page. Its first form also took the layer's one-byte "is this
    page there" reads, 1,000 to 1,300 faults a frame, and won nothing;
    reads under 256 bytes now stay with the kernel and a source that
    faulted is left alone for two seconds (4 to 9 faults a frame). The
    submit thread's kernel time fell from 44 to 20 ms a frame; the
    picture is the same.
  - The layer's submit and copy-back threads ask for the fast cores of a
    hybrid processor (`VICTORY_FAST_CORES=0`: not): the submit thread was
    found on a slow core in 6 % of its looks, now in none of 21,059.
    `VICTORY_DISK_CORES=slow` (off, not measured) asks the slow cores for
    the DirectStorage workers, which are as many as the logical
    processors and free to run anywhere.
  - Behind switches, built and unit-tested, not yet shown to pay:
    `VICTORY_REFRESH_ONCE=1`, `VICTORY_VIEW_KNOWN=1`,
    `VICTORY_BACK_OWN_VIEW=1` (picture to be checked),
    `VICTORY_ASYNC_PRESENT=1` (the present goes behind the queue's line;
    no gain while the card is the limit).
  - Tried and refused: `VICTORY_BVH_HOLD=1` holds back the 64 programs
    that walk ray-tracing structures nothing has built; the picture loses
    its indirect light with it, so it stays off.
  - The process opts out of the system's efficiency mode
    (`VICTORY_EFFICIENCY_MODE=1` leaves it); on a hybrid processor the
    frame log found the title's render and recording threads on fast
    cores in every look of a run, and the layer's submit thread on a slow
    core in 6 % of its looks.
  Still open: the per-record clamp is most of the card's time, and the
  copies between the PC's memory and the card's are most of the submit
  thread's; the cause of the one device loss with windows is not known.
  A first play session past the first rooms reached the open world and
  lost the device there with no driver entry (reason "device removed"):
  the card had no room for new textures from the opening cutscene on
  (9.2 of a 10.3 GB budget on a 12 GB card; 2,073 passes made no texture
  for lack of room; 44,613 textures let go and made again in 13 minutes).
  Cause of that removal not known either.
- Known and counted, not fixed yet: an indirect compute pass asks for
  about 77,500 thread groups in x where one dispatch runs 65,535 (the
  rest is cut off).

Memory sync between the title's memory and the copies on the card:
- **A write through one view of a physical page is seen by the copies taken
  through its other views** (`VICTORY_VIEW_GUARD=0` turns it off). The title
  shows one page at several addresses and a copy was guarded at one of them
  only. Measured on the test machine: 0.8 to 1.0 million such writes in a
  five-minute run, every one of which used to leave a copy stale. Since it
  went in, the first room had its walls in the later frames of 4 runs out of
  4 (2 of 5 before); the room's very first shot still comes up without walls
  in some runs.
- Kernel reads into title memory are read beside and copied in, so a block
  guarded again in between no longer fails the read.
- Writes of the storage layer into physical pages are counted where the title
  has those pages mapped (10,000 to 50,000 guarded blocks of copies lay under
  them in every run) and can open the guards there:
  `VICTORY_PAGE_WRITES=guards` (off by default; `1`, `slots`, `all` do more).
- Failed page-destination reads are retried (`VICTORY_DSTORAGE_RETRIES=0` = as before).

Card memory (the test card is full from the first frame of the world: 85 to
99 % of its budget for a whole run, about 26,000 texture copies let go and
3,400 made again in five minutes, 2.8 GB of render targets that are all in
use):
- Levels on demand, **off by default** (`VICTORY_LEVELS_ON_DEMAND=1`): a
  texture the title fills takes card memory only for the levels the title
  holds (2.2 to 3.0 GB of the copies were levels it does not hold). Its test
  runs on the software adapter; it has not run on a card yet.
- Refused texture slots are held back for a few seconds so that a pass
  reaches the slots behind them (`VICTORY_FULL_CARD_PASS=0` = as before);
  room can be kept for the card copies of title buffers
  (`VICTORY_CARD_ROOM_FOR_BUFFERS=1`, off).

Tools and diagnostics:
- `victory translate <shader file> --warm all --jobs N` translates every
  shader ahead of a run, on the processor only (about 4.5 to 6.5 minutes
  here); the shader cache is keyed on the translator's source, so a new
  build that does not touch the translator starts warm.
- Counters, all off or diagnostics-only: `VICTORY_MERGE_RACE=1` (title bytes
  a copy-back is about to put back to older values: none found in 138,000
  copy-backs), `VICTORY_WALLS_TRACE`, `VICTORY_CARD_ACCOUNTING`, and a census
  of the copies the texture sweep never lets go.

### What this fork changed (summary)

Rendering:
- Meshlet culling read its scene buffers as the wrong kind of view, so the world drew almost nothing; fixed.
- Per-draw descriptor tables could run out, after which draws used slots never made for them and the card hung; fixed.
- Light packed by the game's compute shaders was read back through empty copies (sky, probes, fog); fixed for every mip level and layer.
- The fog reconstruction, shadow-mask, hair, blend-shape skinning, fire and several material programs did not translate; fixed.
- Gathers at a named mip level (with and without an offset) read level 0; fixed.
- Implicit-level samples outside pixel shaders produced invalid DXIL in 730 compute programs; fixed.
- Rectangle lists, depth and stencil twins for depth textures written by compute, texture read-backs.
- Ray tracing, in progress and off by default: the game's own structure-builder programs translate, title memory can be read by raw address from shaders, the hardware ray/box and ray/triangle test is emulated in DXIL (checked against a reference on the GPU), and 59 of the 64 trace programs translate. Nothing is lit by it yet. See [docs/fork-ray-tracing.md](docs/fork-ray-tracing.md).
- Shader translation over the title's whole shader pack, RTX 5090 (32-lane waves), `victory translate --validate --pipelines`: 25,682 of 25,953 programs translate, validate and get a pipeline, against 25,452 at 9376fd6 (compute 2,729 of 2,734, mesh 8,694 of 8,765, pixel all 12,067, vertex all 2,192). What is left: the 195 unused voxel geometry stages, five ray-trace programs, and 71 mesh programs as translated alone (linked for their pipelines, below).
- Console wave64 compute programs on a GPU with 32-lane waves run in a paired form where the host refused them (they asked for 64-lane waves) or where only an approximate per-half form took them: one host wave for each console wave, each thread two of its lanes. The screen-space tracing tiles, particles, decal trails, hair shadow masks and three structure-builder programs get pipelines, and 36 programs in which each half-wave counted, ORed or elected over its own 32 lanes run as on the console, at about 2.9 times their DXIL size (not timed in a run yet). `VICTORY_WAVE_PAIRS=0` turns it off.
- Mesh programs whose outputs pass D3D12's 32 KiB as translated alone (71: pedestrian, vehicle and prop transforms) are linked with the pixel program of each pipeline, so that the mesh program writes only what that program reads (`VICTORY_MESH_LINK=0` turns it off). Offline, 710 such pairs link, validate and get a pipeline.
- Also translated now: a float beside an integer or the front face in one signature register (one interpolation mode per register, rows in the validator's order), immediate samplers chosen by a uniform branch, pixel programs ordered by primitive (on rasterizer-ordered views), whole-wave sections that a jump splits, 64-bit floats and their wave sums, descriptors read only for their size, a subtraction on group-shared memory (NVIDIA refuses the atomic; a compare-exchange loop). Loops of ray-testing programs stop at 1,024 iterations while the builder is off (shadow mask 11.7 → 2.4 ms a frame).

Speed and memory:
- Waits for memory values, write-watch queries and page queries no longer walk page tables per call.
- Large title buffers get a copy in card memory, with shader writes found by hashing on the card.
- Submissions can be sent ahead of the wait for the card (`VICTORY_LATE_WAIT`), with a relay thread that lands results as the card finishes (`VICTORY_RELAY`): in a first five-minute run the card's idle time behind the submitting thread fell from about 136 to 30 ms per frame.
- Title memory can be opened as a graphics heap so that buffers are used in place instead of copied (`VICTORY_INPLACE`, first stage only; a probe confirmed the approach on the test GPU).
- The bindless texture queue no longer uploads stale views; the view table no longer grows without bound; the pipeline library is mapped instead of read into memory.
- An intermittent device loss was traced to the pipeline library file being replaced while mapped; fixed.

- Translated shaders get clean-up passes over the finished module (dead code, constant folding, reuse of values, branch facts); the pack's host instruction count falls by about 43 %, worth about 20 ms of GPU time per frame on the test machine (`VICTORY_SHADER_OPT=0` turns them off).

Most of the newer speed changes sit behind environment switches that are off
by default (`VICTORY_LATE_WAIT`, `VICTORY_RELAY`, `VICTORY_WIDE_BACK`,
`VICTORY_CARD_BUFFERS_MIN_MB`, `VICTORY_INPLACE`, `VICTORY_BVH_BUILDER`, ...).
Each is described in a comment where it is read.

### Known problems

- Everything in shadow is black (see above).
- The interior of the first gameplay room is sometimes not drawn after the cutscene hands over to gameplay.
- Around 24 GB of memory is in use during play.
- A few style lints are switched off for this fork in the workspace `Cargo.toml` until the newer code is tidied.
- Many changes here are specific to this title and this GPU, and some are marked `shortcut:` in the code where they approximate what the console does.

Everything below this line is upstream VIctory's own README, unchanged; its
status section describes upstream, not this fork.

---

# VIctory

Runs the title (Xbox Series X|S) natively on Windows, from files
the user supplies.

The title's x86-64 code runs unchanged inside our process; VIctory supplies
the platform underneath it and converts the title's data, ahead of time where
it can and on first use where it must (shaders). See
[docs/architecture.md](docs/architecture.md).

## Status (2026-11-23)

**Not playable yet.** The title boots, runs its menus at its own 30 fps and
loads the world, where rendering fails.

| Stage | State |
|-------|-------|
| Boot, first-boot screens (accessibility, language, settings, legal) | works, 30 fps |
| Intro video with sound | works |
| Start screen, profile and saves | works |
| Loading into the world (about 50 s) | works |
| The world (first mission's intro) | reached and runs with indirect draws; its title card renders, the scene after it is still dark; 0.5-1.8 s per frame |

Tested on one machine: Windows 11, AMD RX 6700 XT.

> **Warning.** Entering the world on a real GPU can currently hang it, and
> Windows then resets the display driver (every monitor goes black for a
> moment). To look further, use a diagnostics build with
> `VICTORY_D3D12_WARP=1`, which renders on the software adapter.

What each part does today and what it still needs:

| Subsystem | Today | Needs |
|-----------|-------|-------|
| Loader | Runs the title's executable and its DLLs in our process; crash reports name the faulting code and are kept in `crashes.log` | – |
| Runtime services | One signed-in offline user, licences, packages, saves that survive a kill at any step, text entry dialog | Speech is silent; the dialog and relaunch are untested live |
| Network | Offline by design: the title cannot reach the network | – |
| Files and streaming | Archives and streamed data read, zlib and BCPack decoded in parallel | Loading is limited by writes into title memory |
| Input | XInput pads, keyboard and mouse as a pad (rebindable) | Trigger rumble; keyboard prompts |
| Audio | Menu, video and loading sound; XMA voices and Opus streams decode | Reverb is silent; Opus results unverified against the title |
| GPU: menus | Draws, compute and presentation on host D3D12; only changed title memory is copied | Frame pacing drifts on one screen in release builds |
| GPU: world | Mesh pipelines, shader-written buffers, descriptor lifetimes, indirect execution (arguments clamped), pipelines made by worker threads and kept on disk | Most of the scene is still missing (untranslated shaders drop draws); frames take seconds; real queries, depth bounds, HDR and scaling are missing |
| Shaders | Console GPU code recompiled to DXIL at first use, cached on disk. Menus: all. World: mesh 8,191 of 8,506, vertex/pixel 10,617 of 12,346, compute 1,643 of 2,539 | 64-bit image atomics, merged geometry stages, the ray-tracing builder's shaders; some compute programs need a GPU with 64-lane waves |
| Textures | Console tiled layouts and detiling for the modes the title uses | Render-target metadata planes |
| Series S mode | Runs through the menus to the loading screen | The world, as above |

## Getting started

Requirements: Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with
shader model 6.6 (and mesh shaders for the world), Rust (pinned in
`rust-toolchain.toml`).

The title also needs several GiB of available Windows commit at startup.
Check **Committed** under **Task Manager → Performance → Memory**. If
`crashes.log` reports Windows error 1455, close other applications or increase
the Windows paging file size, then retry.

```powershell
cargo xtask install-hooks                # asset-policy check on every commit
copy victory.example.toml victory.toml   # then set your paths (outside the repo)
cargo test --workspace                   # needs no game data
cargo run -p victory -- config           # check your paths
```

Run the title:

```powershell
cargo build --release -p victory
target\release\victory.exe run <game dir>\<the title's exe> --execute
```

The window caption shows the framerate. Saves, settings and `crashes.log`
are kept under the data folder (default `%LOCALAPPDATA%\victory\data`), derived
data under the cache folder.

Input: XInput pads, and keyboard and mouse played like a PC game. The title
reads pads only, so the keyboard and mouse act on the first connected pad (or
a stand-in pad when none is): both work at once, each stick follows whichever
moved it last. The mouse turns the camera through the right stick, its motion
turned into a rate at each read of the title so the same movement turns the
same angle at any frame rate. While the window has focus the cursor is hidden
and held in it; Alt-Tab or Esc frees it, a click or Esc captures it again.

| Keys (on foot) | Pad | Keys (in a vehicle) | Pad |
|------|-----|------|-----|
| W A S D, Left Alt | left stick, walk | W / S, A D | RT / LT, steer |
| mouse, left / right button | right stick, RT / LT | Space, E | RB, LS |
| Left Shift, Space, F, R | A, X, Y, B | F, G | Y + vehicle layout off, layout only |
| Q, E, Left Ctrl, C | RB, D-pad right, LS, RS | | |
| Tab, V or M, wheel | LB, View, D-pad left / right | | |
| arrows, Enter, Backspace, Esc or P | D-pad, A, B, Menu | | |

The letters are the keys at those places on a US keyboard, so the left stick
is Z Q S D on an AZERTY keyboard (`keys_by_position = false`: by letter).
F (enter a vehicle) also switches to the vehicle layout and back; G switches
it alone when it is out of step. Rebind everything in `[controls]` of
`victory.toml` (see `victory.example.toml`); `enabled = false` brings back the
old fixed stand-in pad (W A S D / I J K L sticks, Enter, Backspace, X, Y).

## Commands

```
victory inspect <exe|dll> [--imports] [--exports]
victory census [paths...] [--missing] [--strict]
victory lib-exports <import.lib> [--list]
victory shader-info [paths...] [--verbose] [--disasm]
victory translate [paths...] [--validate] [--pipelines] [--list] [--jobs N]
victory run <exe> [--execute] [--unimplemented trap|zero] [--strict]
            [--console series-x|series-s] [-- <title arguments>]
victory config
```

Logging: `VICTORY_LOG=victory_loader=trace,info` (a release build logs
info and above).

## Debugging

A release build has no debugging aids. Build them in with

```powershell
cargo build --profile diag -p victory --features diagnostics   # target\diag\victory.exe
```

| Variable | Effect |
|----------|--------|
| `VICTORY_LOG=<target>=debug,...` | debug and trace output |
| `VICTORY_STACKS=20,60` | every thread's stack at those seconds |
| `VICTORY_WATCH=<rva>,...` | logs a title u32 when it changes |
| `VICTORY_KEYS=30:Enter,35:X` | presses keys at those seconds, focus or not |
| `VICTORY_CRASH_SAVE=<rva>:<len>,...` | saves title memory to the cache folder on a crash |
| `VICTORY_CAPTURE=<dir>` | saves presented frames (powers of two, or `VICTORY_CAPTURE_FRAMES=<list>`) |
| `VICTORY_DUMP_TARGETS=1` | with the two above, saves every render target of those frames |
| `VICTORY_D3D12_DEBUG=1` | host debug layer |
| `VICTORY_D3D12_GBV=1` | with the above, GPU-based validation (bad descriptor use reported, not faulted; slow) |
| `VICTORY_D3D12_DRED=1` | on a GPU failure, the allocation hit and the commands in flight |
| `VICTORY_EARLY_PIPELINES=0` | start a host pipeline when a draw or dispatch first requests it. Keeps the learned list, shader cache, and driver pipeline library for comparing startup scheduling across runs |
| `VICTORY_PIPELINE_WAIT_MS=24` | reduce the per-frame wait for background pipeline creation from the default of 500 ms. A draw can be omitted until its pipeline is ready |
| `VICTORY_SKIP_PIPELINES=<hash>,...` | omit draws and dispatches, including indirect ones, whose pipeline contains a listed 16-digit shader hash |
| `VICTORY_GPU_TIMING=1` | with the above, each list of a submission run and timed on its own; slow lists logged with their slowest commands |
| `VICTORY_D3D12_WARP=1` | host device on the software adapter (slow; cannot reset the driver) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | translated shader loops stop after `n` iterations per lane instead of 2^20 (own shader cache entries); if a lower `n` makes world frames much faster, loops hitting the limit are what makes them slow |
| `VICTORY_MESH_LINK=0` | a mesh pipeline whose mesh program's outputs pass 32 KiB is made of the shared translations, as before, not of the pair linked for it |
| `VICTORY_AUDIO_CAPTURE=<file.wav>` | records what the title plays (our process only) |
| `VICTORY_XMA_DUMP=<dir>` | saves the first 32 XMA voices' context and input |

Every build appends crashes and host GPU failures to `crashes.log` in the
title's data folder.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
