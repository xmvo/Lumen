@echo off
rem Starts the title with the settings measured on an 8 GB NVIDIA card
rem (RTX 3070 8 GB, Ryzen 7 5700, 32 GB RAM, Windows 10 22H2 with the Agility
rem SDK 1.620 runtime in a D3D12 folder next to victory.exe): about 9 fps in
rem the first room instead of 2.7 (8.5 with the two fixes for the picture). See docs/notes-8gb-nvidia.md.
rem
rem Usage: set VICTORY_GAME_EXE to the title's executable in your own dump, then
rem   scripts\play-8gb-nvidia.cmd [--console series-x] [more run options]
setlocal
if not defined VICTORY_GAME_EXE (
  echo Set VICTORY_GAME_EXE to the title's executable in your own dump. 1>&2
  exit /b 2
)
set "runner=%~dp0..\target\release\victory.exe"
if not exist "%runner%" (
  echo Build it first: cargo build --release --locked -p victory 1>&2
  exit /b 2
)
rem Series S unless a console is named: the same picture here, faster.
set "CONSOLE=--console series-s"
echo %* | find /i "--console" >nul && set "CONSOLE="
rem Without it the title stops itself about 10 minutes in.
if not defined VICTORY_NO_WATCHDOG set "VICTORY_NO_WATCHDOG=1"
rem Title buffers' copies of 256 KiB and more on the card (up to 1.5 GB).
rem On cards of 12 GB or less only those of 16 MiB and more are by default;
rem the rest live in the PC's memory, where the card's shaders write them over
rem the bus and the processor compares them for changes every frame.
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=256"
if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=1536"
rem The texture pool reuses a texture's memory while the card may still draw
rem from it: colourful noise on arms and faces. Off: no noise, no speed lost.
if not defined VICTORY_TEXTURE_POOL set "VICTORY_TEXTURE_POOL=0"
rem Send-ahead stage 2, not the default 3: stage 3 also stops settling at each
rem present, and stretched triangles show for a frame. Stage 2: none, 8.5 fps.
if not defined VICTORY_LATE_WAIT set "VICTORY_LATE_WAIT=2"
rem The processor-side switches of dist\windows-x64\Play-VIctory.cmd, without
rem its two ambient light switches and levels on demand (on this card those
rem made the picture dark and foggy and the card slower).
if not defined VICTORY_TEXTURE_NO_WAIT set "VICTORY_TEXTURE_NO_WAIT=1"
if not defined VICTORY_RELAY set "VICTORY_RELAY=2"
if not defined VICTORY_WIDE_BACK set "VICTORY_WIDE_BACK=1"
if not defined VICTORY_CARD_REHASH_RUNS set "VICTORY_CARD_REHASH_RUNS=1"
if not defined VICTORY_JOIN_COPIES set "VICTORY_JOIN_COPIES=1"
if not defined VICTORY_INDIRECT_WINDOW set "VICTORY_INDIRECT_WINDOW=1"
if not defined VICTORY_RELAY_COPY set "VICTORY_RELAY_COPY=1"
if not defined VICTORY_CARD_STEPS set "VICTORY_CARD_STEPS=2"
if not defined VICTORY_INDIRECT_BATCH set "VICTORY_INDIRECT_BATCH=2"
if not defined VICTORY_MAP_THREADS set "VICTORY_MAP_THREADS=1"
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=1"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"
"%runner%" run "%VICTORY_GAME_EXE%" --execute %CONSOLE% %*
exit /b %errorlevel%
