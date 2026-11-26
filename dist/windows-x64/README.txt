VIctory pareto-residency for Windows x64

Extract the entire ZIP to a folder. From PowerShell in that folder, run:

    $env:VICTORY_GAME_DIR = 'D:\Games\MyDump'
    .\Play-VIctory.cmd pareto-residency

VICTORY_GAME_DIR must contain your own complete dump, including game_scarlett_master.exe and its data and title DLLs. The first run builds a local cache. To use a warmed cache, set VICTORY_CACHE_DIR to an existing directory outside the source repository. Saves use the emulator's normal data directory unless VICTORY_DATA_DIR is set.

Requires Windows x64, an AVX2 CPU, and a Direct3D 12 GPU with Shader Model 6.6 and mesh shaders. The launcher selects Series S with the residency settings from codec_d's build. The included vcruntime140.dll is Microsoft's x64 Visual C++ runtime (Microsoft Corporation), supplied from Visual Studio's redistributable directory. VIctory's license and its format table notice are included.

Source: https://github.com/codec-d/cool_fox/tree/f4b48ad766ddbec1a1867022cd42f7645b0dd55e
