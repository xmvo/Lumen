use anyhow::{Context, bail};
use std::ffi::c_void;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn run(warm: bool) -> anyhow::Result<()> {
    let launcher = std::env::current_exe()?;
    let folder = launcher.parent().context("Launcher folder is missing")?;
    let runtime = folder.join("victory.exe");
    let mut args = std::env::args_os().skip(1).peekable();
    let (mut warp, mut rt) = (false, false);
    while let Some(option) = args.peek() {
        if option == "--warp" {
            warp = true;
        } else if option == "--rt" {
            rt = true;
        } else {
            break;
        }
        args.next();
    }
    let game = match args.next() {
        Some(path) => PathBuf::from(path),
        None => {
            let config = folder.join("game-path.txt");
            let value = fs::read_to_string(&config)
                .with_context(|| format!("Put the game executable path in {}", config.display()))?;
            folder.join(value.trim())
        }
    };
    if args.next().is_some() {
        bail!("Usage\n{} [--rt] [--warp] [game executable]", launcher.display());
    }
    if !game.is_file() || !runtime.is_file() {
        bail!(
            "Game or VIctory executable is missing.\n\n{}\n{}",
            game.display(),
            runtime.display()
        );
    }
    let game = std::path::absolute(game)?;
    let logs = folder.join("logs");
    let state = folder.join("state");
    let cache = std::env::var_os("VICTORY_CACHE_DIR")
        .filter(|v| !v.is_empty())
        .map_or_else(|| folder.join("cache"), |value| folder.join(value));
    fs::create_dir_all(&logs)?;
    fs::create_dir_all(&state)?;
    fs::create_dir_all(&cache)?;
    let _session = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(state.join("launcher.lock"))
        .context("Could not lock the launcher state. Check for another DexNom Runtime session.")?;
    let started = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let action = if warm { "warm" } else { "run" };
    let path = logs.join(format!("{action}-{started}-{}.log", std::process::id()));
    let mut log = OpenOptions::new()
        .append(true)
        .create_new(true)
        .open(&path)?;
    writeln!(
        log,
        "Runtime {}\nGame {}",
        runtime.display(),
        game.display()
    )?;
    let latest = if warm {
        "latest-warm.txt"
    } else {
        "latest.txt"
    };
    fs::write(logs.join(latest), path.display().to_string())?;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut child = Command::new(&runtime);
    if rt {
        child
            .env("VICTORY_BVH_BUILDER", "1")
            .env("VICTORY_BVH_QUARANTINE", "0")
            .env("VICTORY_ACTIVATE_RAYS", "1")
            .env("VICTORY_RT_RESOLVE", "1");
        writeln!(log, "Experimental ray tracing requested")?;
    }
    // Without rays, what the sun does not reach is black (faces, clothes,
    // rooms): a constant stands in for the indirect light unless the
    // caller chose a level or turned it off (`VICTORY_AMBIENT=off`).
    if !rt && std::env::var_os("VICTORY_AMBIENT").is_none() {
        child.env("VICTORY_AMBIENT", "0.6");
        writeln!(log, "Indirect light: a constant of 0.6 (VICTORY_AMBIENT)")?;
    }
    if warp {
        child.env("VICTORY_D3D12_WARP", "1");
    }
    if std::env::var_os("VICTORY_DEBUG_HUD").is_none() {
        child.env("VICTORY_DEBUG_HUD", "1");
    }
    child
        .current_dir(folder)
        .env("VICTORY_DATA_DIR", &state)
        .env("VICTORY_CACHE_DIR", &cache)
        .stdin(Stdio::null())
        .stderr(log.try_clone()?)
        .creation_flags(CREATE_NO_WINDOW);
    let status = if warm {
        preload(&mut child, &game, &cache, &path, &mut log)?
    } else {
        let status = child
            .arg("run")
            .arg(&game)
            .args([
                "--execute",
                "--console",
                "series-x",
                "--unimplemented",
                "trap",
            ])
            .stdout(log.try_clone()?)
            .status()
            .with_context(|| format!("Could not start {}", runtime.display()))?;
        writeln!(log, "VIctory exited with {status}")?;
        status
    };
    if !status.success() {
        bail!(
            "VIctory exited with {status}.\n\nLog file\n{}",
            path.display()
        );
    }
    Ok(())
}

fn preload(
    child: &mut Command,
    game: &Path,
    cache: &Path,
    path: &Path,
    log: &mut File,
) -> anyhow::Result<ExitStatus> {
    let root = game.parent().context("Game install folder is missing")?;
    writeln!(
        log,
        "Scan {}\nCache {}\nThe offline reader recognizes SGD5 records and raw DXBC in files up to 2 GiB. Graphics pipelines are created when the game supplies their draw state.",
        root.display(),
        cache.display()
    )?;
    println!(
        "Preloading shaders and compute pipelines from {}\nLive log: {}",
        root.display(),
        path.display()
    );
    let mut worker = child
        .arg("warm")
        .arg(root)
        .stdout(Stdio::piped())
        .spawn()
        .context("Could not start DexNom Runtime shader preload")?;
    let stdout = worker
        .stdout
        .take()
        .context("Warmup output pipe is missing")?;
    let last =
        BufReader::new(stdout)
            .lines()
            .try_fold(None, |summary, line| -> anyhow::Result<_> {
                let line = line?;
                writeln!(log, "{line}")?;
                let _ = writeln!(std::io::stdout().lock(), "{line}");
                Ok(if line.starts_with("warm ") {
                    Some(line)
                } else {
                    summary
                })
            });
    let status = worker.wait()?;
    writeln!(log, "VIctory exited with {status}")?;
    if status.success()
        && let Err(message) = check_warm_summary(last?.as_deref())
    {
        writeln!(log, "{message}")?;
        bail!("{message}\n\nLog file\n{}", path.display());
    }
    Ok(status)
}

/// The runtime prints this after saving its pipeline library. Preserve its exact
/// counters in the log, including partial refusal; an empty scan is not a preload.
fn check_warm_summary(summary: Option<&str>) -> Result<(), &'static str> {
    let Some(summary) = summary else {
        return Err("VIctory did not report a shader warmup result");
    };
    let mut words = summary.split_whitespace();
    if words.next() != Some("warm")
        || words
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .is_none_or(|n| n == 0)
    {
        return Err("No shaders successfully translated. See the runtime totals in the log");
    }
    Ok(())
}

#[allow(dead_code)] // the console warmer reports errors to stderr
pub(super) fn show_error(text: &str) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(owner: *mut c_void, text: *const u16, title: *const u16, flags: u32) -> i32;
    }
    let text: Vec<_> = text.encode_utf16().chain([0]).collect();
    let title: Vec<_> = "DexNom Runtime".encode_utf16().chain([0]).collect();
    // SAFETY: the two strings are NUL terminated and live for the call.
    unsafe { MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10) };
}

#[cfg(test)]
mod tests {
    use super::check_warm_summary;

    #[test]
    fn empty_and_incomplete_scans_are_reported() {
        assert!(check_warm_summary(None).is_err());
        assert!(
            check_warm_summary(Some(
                "warm  0 translated, 0 refused; 0 compute pipelines, 0 refused; 0 s"
            ))
            .is_err()
        );
        assert!(
            check_warm_summary(Some(
                "warm  0 translated, 2 refused; 0 compute pipelines, 0 refused; 1 s"
            ))
            .is_err()
        );
        assert!(
            check_warm_summary(Some(
                "warm  1 translated, 2 refused; 3 compute pipelines, 4 refused; 5 s"
            ))
            .is_ok()
        );
    }
}
