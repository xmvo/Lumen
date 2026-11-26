//! Lumen launcher.
//!
//! Sits beside `bin\lumen.exe` (the compatibility layer), `shaders\` (the shader containers of the
//! user's own copy of the game) and `art\` (backdrop pictures, loading screens captured from that
//! same copy). Pressing Play prepares the shader cache once for each build, then starts the game.
//! A running game is never closed from here: only the shader step stops with the window.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Instant, UNIX_EPOCH};

use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// What the game commits at most, with some room (measured: 21.7 GB when a run ran out).
const GAME_NEEDS_GB: f64 = 26.0;

/// The switches the ray tracing build runs with: the set its authors recommend (pull requests
/// #814 and #905 of fxceless/victory-fork, run by them on an RTX 3080).
const RAY_SWITCHES: [(&str, &str); 5] = [
    ("VICTORY_BVH_BUILDER", "1"),
    ("VICTORY_BVH_QUARANTINE", "0"),
    ("VICTORY_CARD_FETCH_INFLIGHT", "1"),
    ("VICTORY_TEXTURE_DROP_MIPS", "1"),
    ("VICTORY_TEXTURE_ROOM_PERCENT", "82"),
];

/// The switches the NVIDIA build runs with (the same set the Desktop launcher used).
const GAME_SWITCHES: [(&str, &str); 14] = [
    ("VICTORY_NO_WATCHDOG", "1"),
    ("VICTORY_ASYNC_SUBMIT", "1"),
    ("VICTORY_QUEUE2", "1"),
    ("VICTORY_SKIP_WC_COMPARE", "1"),
    ("VICTORY_LEVELS_ON_DEMAND", "1"),
    ("VICTORY_WIDE_TEXTURES", "1"),
    ("VICTORY_CARD_BUFFERS_MB", "1024"),
    ("VICTORY_COPY_PLAIN", "1"),
    ("VICTORY_MERGE_GAP", "0"),
    ("VICTORY_WIDE_REFRESH", "1"),
    ("VICTORY_WIDE_THREADS", "4"),
    ("VICTORY_JOIN_COPIES", "1"),
    ("VICTORY_CARD_REHASH_RUNS", "1"),
    ("VICTORY_WIDE_MERGE", "1"),
];

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
struct Settings {
    game_path: String,
    lit_shade: bool,
    level: f64,
    fast: bool,
    /// Experimental: run the ray tracing build (`bin\lumen-rt.exe`) instead of the usual one.
    rays: bool,
    /// With rays: the Series X profile (full resolution) instead of Series S. Needs far more memory.
    rays_full: bool,
    /// The game waits for a pipeline that is still being built instead of leaving its draws out:
    /// nothing is missing from the picture, a first sight freezes for a moment instead.
    wait_pipelines: bool,
    /// The fill light also goes to the five other light images the game's resolve writes (skin and
    /// outlines read them). New and not yet seen in a run: off gives the picture of before.
    skin_light: bool,
    /// Every texture is uploaded whole at once instead of its small levels first. Sharp textures
    /// arrive sooner; it takes far more memory (for PCs with 32 GB or more).
    full_textures: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            // picked in Settings: the executable of the user's own copy
            game_path: String::new(),
            lit_shade: true,
            level: 0.6,
            fast: true,
            rays: false,
            rays_full: false,
            wait_pipelines: true,
            skin_light: true,
            full_textures: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Idle,
    Prep,
    Game,
}

struct Inner {
    phase: Phase,
    child: Option<Child>,
    log: PathBuf,
    play_after: bool,
    total: usize,
    started: Instant,
    step: String,
    hint: String,
    /// A hint that stays until the next action (why the game closed, what the shader step did).
    sticky: bool,
    progress: f64,
    prepared: bool,
    shader_files: bool,
    polls: u64,
    third_label: String,
    third_value: String,
    /// How far the shader step is, in words.
    detail: String,
}

struct App {
    root: PathBuf,
    inner: Mutex<Inner>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    phase: &'static str,
    step: String,
    hint: String,
    progress: f64,
    shaders_ready: bool,
    /// Whether there are shader files to prepare ahead (none: they are translated as the game runs).
    shader_files: bool,
    free_gb: f64,
    total_gb: f64,
    low_memory: bool,
    third_label: String,
    third_value: String,
    detail: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Init {
    settings: Settings,
    art: Vec<String>,
    /// When the build was made, in seconds since 1970; none if there is no build.
    build: Option<u64>,
}

// ------------------------------------------------------------------ small helpers

#[repr(C)]
struct MemoryStatus {
    length: u32,
    load: u32,
    total_phys: u64,
    avail_phys: u64,
    total_page: u64,
    avail_page: u64,
    total_virtual: u64,
    avail_virtual: u64,
    avail_extended: u64,
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalMemoryStatusEx(buffer: *mut MemoryStatus) -> i32;
}

/// Free memory, installed memory, and what Windows can still hand out (RAM and page file), in GB.
fn memory() -> (f64, f64, f64) {
    let mut status = MemoryStatus {
        length: std::mem::size_of::<MemoryStatus>() as u32,
        load: 0,
        total_phys: 0,
        avail_phys: 0,
        total_page: 0,
        avail_page: 0,
        total_virtual: 0,
        avail_virtual: 0,
        avail_extended: 0,
    };
    // SAFETY: the buffer is a properly sized MEMORYSTATUSEX with its length field set.
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return (0.0, 0.0, 0.0);
    }
    let gb = 1024.0 * 1024.0 * 1024.0;
    (
        (status.avail_phys as f64 / gb * 10.0).round() / 10.0,
        (status.total_phys as f64 / gb).round(),
        (status.avail_page as f64 / gb * 10.0).round() / 10.0,
    )
}

/// The folder that holds `bin\lumen.exe`: the launcher's own folder, or one above it (when the
/// launcher runs from its build folder).
fn find_root() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    let mut at = exe.parent().map(Path::to_path_buf).unwrap_or_default();
    let first = at.clone();
    for _ in 0..7 {
        if at.join("bin").join("lumen.exe").is_file() {
            return at;
        }
        match at.parent() {
            Some(up) => at = up.to_path_buf(),
            None => break,
        }
    }
    first
}

fn count_shaders(folder: &Path) -> usize {
    fs::read_dir(folder)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("dxbc")))
                .count()
        })
        .unwrap_or(0)
}

fn modified_secs(path: &Path) -> Option<u64> {
    let time = fs::metadata(path).ok()?.modified().ok()?;
    Some(time.duration_since(UNIX_EPOCH).ok()?.as_secs())
}

/// The end of a log as text, without colour codes.
fn read_tail(path: &Path, bytes: u64) -> String {
    let Ok(mut file) = fs::File::open(path) else { return String::new() };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let take = len.min(bytes);
    if file.seek(SeekFrom::Start(len - take)).is_err() {
        return String::new();
    }
    let mut raw = Vec::with_capacity(take as usize);
    let _ = file.take(take).read_to_end(&mut raw);
    let text = String::from_utf8_lossy(&raw);
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            for d in chars.by_ref() {
                if d.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The number written just before the last `marker` in `text` ("1234 shaders so far", "5.8 fps, ").
fn number_before(text: &str, marker: &str) -> Option<String> {
    let at = text.rfind(marker)?;
    let head = text[..at].trim_end();
    let start = head
        .rfind(|c: char| !(c.is_ascii_digit() || c == '.'))
        .map(|i| i + 1)
        .unwrap_or(0);
    let number = &head[start..];
    (!number.is_empty()).then(|| number.to_string())
}

fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

// ------------------------------------------------------------------ the launcher itself

impl App {
    fn exe(&self) -> PathBuf {
        self.root.join("bin").join("lumen.exe")
    }
    fn shaders(&self) -> PathBuf {
        self.root.join("shaders")
    }
    fn logs(&self) -> PathBuf {
        let logs = self.root.join("logs");
        let _ = fs::create_dir_all(&logs);
        logs
    }

    fn settings(&self) -> Settings {
        fs::read_to_string(self.root.join("settings.json"))
            .ok()
            .and_then(|text| serde_json::from_str(text.trim_start_matches('\u{feff}')).ok())
            .unwrap_or_default()
    }

    /// What the shader step reads, and how many shaders it holds if that is known: the game's own
    /// shader archive beside the game file (every shader the game has, about 26 thousand), else
    /// a `shaders` folder of containers beside the launcher.
    fn shader_source(&self) -> Option<(PathBuf, usize)> {
        let game = self.settings().game_path;
        if let Some(archive) = Path::new(&game).parent().map(|d| d.join("shaders.rpf")).filter(|p| p.is_file()) {
            return Some((archive, 0));
        }
        let folder = self.shaders();
        let count = count_shaders(&folder);
        (count > 0).then_some((folder, count))
    }

    /// Names this build and this set of shaders: the shader step is done once for each.
    fn stamp(&self) -> String {
        let exe = self.exe();
        let len = fs::metadata(&exe).map(|m| m.len()).unwrap_or(0);
        let shaders = match self.shader_source() {
            Some((path, 0)) => format!(
                "all-{}-{}",
                fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                modified_secs(&path).unwrap_or(0)
            ),
            Some((_, count)) => count.to_string(),
            None => "0".into(),
        };
        format!("{len}-{}-{shaders}", modified_secs(&exe).unwrap_or(0))
    }

    fn prepared(&self) -> bool {
        fs::read_to_string(self.root.join("prepared.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(text.trim_start_matches('\u{feff}')).ok())
            .and_then(|v| v.get("stamp").and_then(|s| s.as_str()).map(|s| s == self.stamp()))
            .unwrap_or(false)
    }

    /// The experimental ray tracing build: another layer, kept beside the usual one.
    fn ray_exe(&self) -> PathBuf {
        self.root.join("bin").join("lumen-rt.exe")
    }

    /// Whether a layer is running already (started from here or from anywhere else): Windows
    /// refuses to open a running program's file for writing.
    fn running(&self) -> bool {
        [self.exe(), self.ray_exe()]
            .iter()
            .any(|exe| exe.is_file() && fs::OpenOptions::new().write(true).open(exe).is_err())
    }

    fn spawn(&self, args: &[&str], log: &Path, switches: &[(String, String)]) -> std::io::Result<Child> {
        self.spawn_exe(&self.exe(), args, log, switches)
    }

    fn spawn_exe(
        &self,
        exe: &Path,
        args: &[&str],
        log: &Path,
        switches: &[(String, String)],
    ) -> std::io::Result<Child> {
        let out = fs::File::create(log)?;
        let err = out.try_clone()?;
        let mut command = Command::new(exe);
        command
            .args(args)
            .current_dir(self.root.join("bin"))
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err)
            .creation_flags(CREATE_NO_WINDOW);
        // Only the switches chosen here: none left over from the shell the launcher came from.
        for (name, _) in std::env::vars() {
            if name.starts_with("VICTORY_") {
                command.env_remove(name);
            }
        }
        for (name, value) in switches {
            command.env(name, value);
        }
        command.spawn()
    }

    fn start_prep(&self, s: &mut Inner) {
        let Some((shaders, total)) = self.shader_source() else {
            s.play_after = false;
            s.step = "No shader files found".into();
            s.hint = "Pick your game file under Settings: its shaders.rpf is read from the same folder.".into();
            s.sticky = true;
            return;
        };
        s.total = total;
        s.log = self.logs().join("prepare.log");
        let switches = [
            ("VICTORY_HOST_WAVE".to_string(), "32".to_string()),
            ("VICTORY_VIEW_LIMIT".to_string(), "999999".to_string()),
        ];
        let shaders = shaders.to_string_lossy().into_owned();
        match self.spawn(&["translate", "--warm", "all", &shaders], &s.log, &switches) {
            Ok(child) => {
                s.child = Some(child);
                s.phase = Phase::Prep;
                s.started = Instant::now();
                s.progress = 0.0;
                s.sticky = false;
                s.step = "Preparing shaders".into();
                s.detail = if s.total > 0 { format!("{} shaders", thousands(s.total)) } else { "every shader of the game".into() };
                s.hint = "Done once for each build: every shader is made now, so none is made while you play. A few minutes."
                    .into();
            }
            Err(e) => {
                s.play_after = false;
                s.step = "The shader step did not start".into();
                s.hint = e.to_string();
                s.sticky = true;
            }
        }
    }

    fn start_game(&self, s: &mut Inner) {
        let settings = self.settings();
        if !Path::new(&settings.game_path).is_file() {
            self.to_idle(s, "Game file not found", "Pick it under Settings.");
            return;
        }
        let mut switches: Vec<(String, String)> =
            GAME_SWITCHES.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        // Helper threads by the processor: half its logical cores, four at least (what the table
        // above says, measured on four cores and eight threads) and sixteen at most (the layer's
        // own limit).
        let helpers = std::thread::available_parallelism().map_or(4, |n| n.get() / 2).clamp(4, 16);
        for (name, value) in switches.iter_mut() {
            if name == "VICTORY_WIDE_THREADS" {
                *value = helpers.to_string();
            }
        }
        if settings.lit_shade {
            switches.push(("VICTORY_AMBIENT".into(), format!("{:.1}", settings.level)));
            if !settings.skin_light {
                switches.push(("VICTORY_AMBIENT_ALSO".into(), "none".into()));
            }
        }
        if settings.full_textures {
            switches.retain(|(name, _)| name != "VICTORY_LEVELS_ON_DEMAND");
        }
        if settings.fast {
            switches.push(("VICTORY_ASYNC_PRESENT".into(), "1".into()));
        }
        if settings.wait_pipelines {
            // up to four seconds a frame for pipelines being built (the layer's own default is half
            // a second, after which the draws of a frame that needed them are left out)
            switches.push(("VICTORY_PIPELINE_WAIT_MS".into(), "4000".into()));
        }
        let secs = std::time::SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        s.log = self.logs().join(format!("game-{secs}.log"));
        // Rays: the other build, with its own switches and none of ours (it does not know them).
        let mut exe = self.exe();
        let mut args = vec!["run", settings.game_path.as_str(), "--execute"];
        let mut loading = "Loading takes about four minutes. The world comes in slowly and can look stuck: it is not.";
        if settings.rays {
            exe = self.ray_exe();
            if !exe.is_file() {
                self.to_idle(s, "Ray tracing build not found", "Expected bin\\lumen-rt.exe. Turn Ray tracing off under Settings.");
                return;
            }
            switches = RAY_SWITCHES.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
            if !settings.rays_full {
                args.extend(["--console", "series-s"]);
            }
            loading = "Ray tracing build (experimental): its own shaders are made on this first run, so it is slow to start, and it runs slower than the usual build.";
        }
        match self.spawn_exe(&exe, &args, &s.log, &switches) {
            Ok(child) => {
                s.child = Some(child);
                s.phase = Phase::Game;
                s.started = Instant::now();
                s.progress = 100.0;
                s.sticky = false;
                s.step = "Starting the game".into();
                s.hint = loading.into();
                s.third_label = "fps".into();
                s.third_value = "–".into();
            }
            Err(e) => self.to_idle(s, "The game did not start", &e.to_string()),
        }
    }

    fn to_idle(&self, s: &mut Inner, step: &str, hint: &str) {
        s.phase = Phase::Idle;
        s.child = None;
        s.play_after = false;
        s.progress = 0.0;
        s.step = step.into();
        s.hint = hint.into();
        s.sticky = !hint.is_empty();
        s.prepared = self.prepared();
    }

    fn tick(&self, s: &mut Inner) {
        s.polls += 1;
        match s.phase {
            Phase::Idle => {
                if s.polls % 12 == 0 {
                    s.prepared = self.prepared();
                    s.shader_files = self.shader_source().is_some();
                }
            }
            Phase::Prep => {
                let tail = read_tail(&s.log, 16 * 1024);
                if let Some(done) = number_before(&tail, " shaders so far").and_then(|n| n.parse::<usize>().ok()) {
                    // the archive's count is not known ahead: it holds about 26 thousand
                    let total = if s.total > 0 { s.total } else { 26_000 };
                    s.progress = (100.0 * done as f64 / total as f64).min(99.0);
                    s.detail = if s.total > 0 {
                        format!("{} of {}", thousands(done), thousands(s.total))
                    } else {
                        format!("{} shaders", thousands(done))
                    };
                }
                let exited = s.child.as_mut().is_none_or(|c| !matches!(c.try_wait(), Ok(None)));
                if exited {
                    let summary = tail.rfind(" translated now, ").and_then(|at| {
                        let start = tail[..at].rfind(": ").map(|i| i + 2).unwrap_or(0);
                        let end = tail[at..].find(" do not translate").map(|i| at + i + " do not translate".len())?;
                        Some(tail[start..end].to_string())
                    });
                    match summary {
                        Some(summary) => {
                            let record = serde_json::json!({ "stamp": self.stamp(), "summary": summary });
                            let _ = fs::write(self.root.join("prepared.json"), record.to_string());
                            s.child = None;
                            s.prepared = true;
                            if s.play_after {
                                s.play_after = false;
                                self.start_game(s);
                            } else {
                                self.to_idle(s, "Ready", &format!("Shaders prepared: {summary}."));
                            }
                        }
                        None => self.to_idle(s, "The shader step did not finish", "Open the last log under Settings."),
                    }
                }
            }
            Phase::Game => {
                let tail = read_tail(&s.log, 16 * 1024);
                if let Some(fps) = number_before(&tail, " fps, ") {
                    s.third_value = fps;
                }
                let minutes = s.started.elapsed().as_secs() / 60;
                if minutes >= 1 {
                    s.step = format!("Running  ·  {minutes} min");
                }
                let exited = s.child.as_mut().is_none_or(|c| !matches!(c.try_wait(), Ok(None)));
                if exited {
                    let all = read_tail(&s.log, 400 * 1024);
                    let why = if all.contains("window closed") {
                        ""
                    } else if all.contains("Not enough memory resources") || all.contains("0x8007000E") {
                        "It ran out of memory. Close other apps, or turn Fast mode off."
                    } else if all.contains("device removed") || all.contains("host GPU failure") {
                        "The graphics card stopped it. Start it again."
                    } else if all.contains("unhandled exception") {
                        "It crashed. Start it again; open the last log under Settings if it repeats."
                    } else {
                        ""
                    };
                    s.third_label = "Last run".into();
                    s.third_value = format!("{minutes} min");
                    self.to_idle(s, "The game has closed", why);
                }
            }
        }
    }
}

// ------------------------------------------------------------------ commands

#[tauri::command(async)]
fn load(app: State<'_, App>) -> Init {
    let mut pictures: Vec<PathBuf> = fs::read_dir(app.root.join("art"))
        .map(|entries| entries.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    pictures.sort();
    let art = pictures
        .iter()
        .filter_map(|path| {
            let kind = match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                _ => return None,
            };
            let bytes = fs::read(path).ok()?;
            Some(format!("data:{kind};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
        })
        .collect();
    Init { settings: app.settings(), art, build: modified_secs(&app.exe()) }
}

#[tauri::command(async)]
fn save(app: State<'_, App>, settings: Settings) {
    if let Ok(text) = serde_json::to_string_pretty(&settings) {
        let _ = fs::write(app.root.join("settings.json"), text);
    }
}

#[tauri::command(async)]
fn poll(app: State<'_, App>) -> View {
    let mut s = app.inner.lock().unwrap();
    app.tick(&mut s);
    let (free_gb, total_gb, commit_gb) = memory();
    // The game takes about 22-24 GB. What ends a run is not free RAM but what Windows can still
    // hand out in all (RAM plus page file): a run died with 4 GB of that left.
    let short_commit = commit_gb < GAME_NEEDS_GB;
    let low_memory = s.phase == Phase::Idle && (free_gb < 8.0 || short_commit);
    let hint = if s.phase == Phase::Idle && !s.sticky {
        if short_commit {
            format!(
                "Windows can hand out only {commit_gb:.0} GB more and the game takes about {GAME_NEEDS_GB:.0}. \
                 Close other apps, or make the page file bigger, or it may end while the world loads."
            )
        } else if low_memory {
            "Little memory free. Close browsers and other apps before you play: the game wants most of it.".to_string()
        } else {
            String::new()
        }
    } else {
        s.hint.clone()
    };
    View {
        phase: match s.phase {
            Phase::Idle => "idle",
            Phase::Prep => "prep",
            Phase::Game => "game",
        },
        step: s.step.clone(),
        hint,
        progress: s.progress,
        shaders_ready: s.prepared,
        shader_files: s.shader_files,
        free_gb,
        total_gb,
        low_memory,
        third_label: s.third_label.clone(),
        third_value: s.third_value.clone(),
        detail: s.detail.clone(),
    }
}

#[tauri::command(async)]
fn play(app: State<'_, App>) {
    let mut s = app.inner.lock().unwrap();
    if s.phase != Phase::Idle {
        return;
    }
    if !app.exe().is_file() {
        app.to_idle(&mut s, "No build found", &format!("Expected {}", app.exe().display()));
        return;
    }
    if app.running() {
        app.to_idle(&mut s, "The game is already running", "Close it first, then press Play.");
        return;
    }
    let game = app.settings().game_path;
    if game.is_empty() || !Path::new(&game).is_file() {
        app.to_idle(&mut s, "Pick your game file first", "Settings, then Change: the game executable of your own copy.");
        return;
    }
    // the ray tracing build keeps its own shader cache: nothing is prepared ahead for it
    if app.settings().rays || app.prepared() || app.shader_source().is_none() {
        app.start_game(&mut s);
    } else {
        s.play_after = true;
        app.start_prep(&mut s);
    }
}

#[tauri::command(async)]
fn prepare(app: State<'_, App>) {
    let mut s = app.inner.lock().unwrap();
    if s.phase != Phase::Idle {
        return;
    }
    if app.running() {
        app.to_idle(&mut s, "The game is running", "Shaders are prepared while the game is closed.");
        return;
    }
    s.play_after = false;
    app.start_prep(&mut s);
}

#[tauri::command(async)]
fn pick_game(app: State<'_, App>) -> Option<String> {
    let mut settings = app.settings();
    let mut dialog = rfd::FileDialog::new()
        .set_title("Pick the game executable of your own copy")
        .add_filter("Game executable", &["exe"]);
    if let Some(folder) = Path::new(&settings.game_path).parent().filter(|p| p.is_dir()) {
        dialog = dialog.set_directory(folder);
    }
    let picked = dialog.pick_file()?.to_string_lossy().into_owned();
    settings.game_path = picked.clone();
    if let Ok(text) = serde_json::to_string_pretty(&settings) {
        let _ = fs::write(app.root.join("settings.json"), text);
    }
    Some(picked)
}

#[tauri::command(async)]
fn open_log(app: State<'_, App>) -> bool {
    let newest = fs::read_dir(app.logs())
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "log"))
                .max_by_key(|p| modified_secs(p).unwrap_or(0))
        })
        .unwrap_or(None);
    match newest {
        Some(path) => Command::new("notepad.exe").arg(path).spawn().is_ok(),
        None => false,
    }
}

#[tauri::command(async)]
fn open_controls(app: State<'_, App>) -> bool {
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    let places = [
        app.root.join("CONTROLS.txt"),
        PathBuf::from(format!("{home}\\Desktop\\GTA VI (VIctory) - CONTROLS.txt")),
    ];
    match places.iter().find(|p| p.is_file()) {
        Some(path) => Command::new("notepad.exe").arg(path).spawn().is_ok(),
        None => false,
    }
}

#[tauri::command]
fn win_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn win_min(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn win_close(window: tauri::Window) {
    let _ = window.close();
}

fn main() {
    let app = App {
        root: find_root(),
        inner: Mutex::new(Inner {
            phase: Phase::Idle,
            child: None,
            log: PathBuf::new(),
            play_after: false,
            total: 0,
            started: Instant::now(),
            step: "Ready".into(),
            hint: String::new(),
            sticky: false,
            progress: 0.0,
            prepared: false,
            shader_files: false,
            polls: 0,
            third_label: "Last run".into(),
            third_value: "–".into(),
            detail: String::new(),
        }),
    };
    app.inner.lock().unwrap().prepared = app.prepared();
    app.inner.lock().unwrap().shader_files = app.shader_source().is_some();
    // For checking the shader step by itself: LUMEN_AUTOTEST=prepare starts it with the window.
    if std::env::var("LUMEN_AUTOTEST").as_deref() == Ok("prepare") {
        let mut s = app.inner.lock().unwrap();
        app.start_prep(&mut s);
    }

    tauri::Builder::default()
        .manage(app)
        .invoke_handler(tauri::generate_handler![
            load,
            save,
            poll,
            play,
            prepare,
            pick_game,
            open_log,
            open_controls,
            win_drag,
            win_min,
            win_close
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed) {
                // The shader step is ours alone and stops with the window (what it did so far
                // stays in the cache). A running game is left alone.
                let app = window.state::<App>();
                let mut s = app.inner.lock().unwrap();
                if s.phase == Phase::Prep {
                    if let Some(child) = s.child.as_mut() {
                        let _ = child.kill();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("the launcher window did not open");
}
