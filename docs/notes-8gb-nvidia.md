# Notes: this build on an 8 GB NVIDIA card

One machine: RTX 3070 8 GB (driver 616.92), Ryzen 7 5700 (PCIe 3.0), 32 GB
RAM, Windows 10 22H2 with the Agility SDK 1.620 runtime in a `D3D12` folder
next to `victory.exe`. This branch is cool_fox's `main` plus three commits for
Windows 10 (the Agility SDK exports, the `VICTORY_NO_WATCHDOG` loader hook, and
saves whose old version another process holds open). Measured 2026-11-26 with
diagnostics builds, in the first room after the hand-off, warm caches; fps is
the median of the last eight five-second summaries with 1,000 draws a frame
or more.

## What to run

`scripts/play-8gb-nvidia.cmd` (set `VICTORY_GAME_EXE` first). It starts the
Series S profile with the settings below.

| Settings | fps | Picture |
|---|---|---|
| Series X, nothing set | 2.7 | the first room lit, walls, faces and posters textured |
| `dist/windows-x64/Play-VIctory.cmd`'s settings (Series S) | 2.1 | dark and foggy; the card at 300-450 ms a frame |
| Series S, nothing set | 4.4 | as Series X |
| Series S, `VICTORY_CARD_BUFFERS_MIN_MB=1 VICTORY_CARD_BUFFERS_MB=1024` | 7.4 | the same |
| Series S, `VICTORY_CARD_BUFFERS_MIN_KB=256 VICTORY_CARD_BUFFERS_MB=1536` | 7.9 | the same |
| the same plus the launcher's processor-side switches (below) | **9.2** (9.5-9.8 in lighter shots, ~8.4 at 7,400 draws) | the same |

The processor-side switches are those of `Play-VIctory.cmd` without
`VICTORY_AMBIENT`, `VICTORY_AMBIENT_AIR` and `VICTORY_LEVELS_ON_DEMAND`:
`TEXTURE_NO_WAIT=1 RELAY=2 WIDE_BACK=1 CARD_REHASH_RUNS=1 JOIN_COPIES=1
INDIRECT_WINDOW=1 RELAY_COPY=1 CARD_STEPS=2 INDIRECT_BATCH=2 MAP_THREADS=1
CARD_AWAKE=1 SHOW_AFTER=1 THIN=1 JOIN_PARTS=8 UPLOAD_AHEAD=1` (each with the
`VICTORY_` prefix). With all of them the card holds 5.4 GB of a 6.9 GB budget.

## Why the card buffers matter on this card

An Nsight Graphics GPU Trace of two frames (Series S, nothing set, 4.4 fps):

- the card's engines active 88-93 % of the time, the asynchronous copy engine
  as much, the graphics front end 31-50 %;
- the shader units' instruction throughput 1.5-2.5 % of peak, video memory
  1.6-2.4 %, L2 2.8-4.6 %; thread groups resident 24-40 % of the time;
- 0.75-1.4 GB a frame written by the card into the PC's memory over PCIe
  (15-17 % of the bus's peak; this processor has PCIe 3.0).

The card was not short of shader power: it waited on the bus. `shadow.rs`
keeps a title buffer's copy in the card's memory only if it is 16 MiB or more
(at most 512 MiB in all) when the card's budget is 12 GiB or less. Smaller
copies live in the PC's memory: the card's shaders write them over the bus
(about 230 MB of shader writes a frame went back into title memory) and the
processor compares them for changes every frame (250 MB a frame, 240 MB of it
copied first; the log says so: "VICTORY_CARD_BUFFERS_MIN_MB=1 would have the
card look there"). With copies of 256 KiB and more on the card, the card
compares them itself and only changed pieces come back; the processor
compares 7.8 MB a frame.

Before that, the built-in tools: `VICTORY_PROFILE` showed no thread near a
full core (4.6 of 16 hardware threads busy, the submitting thread 42 % of
one), and `VICTORY_TIMELINE` showed the card running about 181 ms of work a
frame with about 97 ms idle, which set the frame.

## Where the time goes now (9.2 fps)

Per frame: about 85 ms preparing on the submitting thread (about 30 ms of it
uploading about 110 MB of what the title wrote), about 45 ms copying back on
the relay thread, a few ms of the card's own work by the frame line's count.
Both threads now set the frame together; the card is no longer the limit.

## Tried, worse on this card

- avatarmanz/VIctory-7900XTX `main` at c9c932c with our three commits, Series S,
  its renderer patches 2, 6 and 7: 2.6 fps, the card over its budget (8.1 of
  6.8 GB), stair-step squares on lit surfaces. With patches 1 and 5 as well
  (the card reads the title's constants heap in the PC's memory) the card took
  1.3 s a frame.
- Ray-tracing structure builder (on fxceless PR #994): runs without crashing,
  no visible gain, about a third of the frame rate.

## Getting the trace

Nsight Graphics' command line (`ngfx.exe --activity "GPU Trace Profiler"
--exe <victory.exe> --args "run <title exe> --execute --console series-s"
--start-after-ms 220000 --limit-to-frames 2 --auto-export --output-dir <dir>`)
needs two things here: no `--platform` argument (Qt takes it as its own and
fails to start), and access to the card's performance counters (NVIDIA Control
Panel, Developer, "Allow access to the GPU performance counters to all users",
which takes effect after a restart; until then, run it as administrator).
