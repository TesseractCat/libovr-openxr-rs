mod patch;

use rand::Rng;
use std::path::{Path, PathBuf};

slint::include_modules!();

fn ready(inspection: &patch::Inspection) -> bool {
    (inspection.echo_original.state == patch::FileState::Original
        && inspection.pns_original.state == patch::FileState::Original)
        || (inspection.echo.state == patch::FileState::Original
            && inspection.pns.state == patch::FileState::Original)
}

fn set_not_ready(window: &EchoPatcher) {
    window.set_patch_enabled(false);
    window.set_indicator_color(slint::Color::from_rgb_u8(198, 40, 40));
    window.set_indicator_text("Not ready".into());
}

fn file_status(label: &str, file: &patch::FileInspection) -> String {
    let hash = file.sha256.as_deref().unwrap_or("-");
    format!(
        "{label}: {}\n  {}\n  SHA-256: {hash}\n",
        file.state.text(),
        file.path.display()
    )
}

fn inspect_installation(window: &EchoPatcher, root: PathBuf) {
    let inspection = patch::inspect(&root);
    let is_ready = ready(&inspection);
    let mut status = format!("Game directory: {}\n\n", inspection.root.display());
    status.push_str(&file_status("echovr.exe", &inspection.echo));
    status.push_str(&file_status("pnsovr.dll", &inspection.pns));
    status.push_str(&file_status(
        "echovr.exe.original",
        &inspection.echo_original,
    ));
    status.push_str(&file_status(
        "pnsovr.dll.original",
        &inspection.pns_original,
    ));
    status.push_str(if is_ready {
        "\nReady: Patch will reconstruct the live files from verified originals."
    } else {
        "\nNot ready: supported original files are required."
    });
    window.set_patch_enabled(is_ready);
    let already_patched = inspection.echo.state == patch::FileState::Patched
        && inspection.pns.state == patch::FileState::Patched;
    if is_ready && already_patched {
        window.set_indicator_color(slint::Color::from_rgb_u8(245, 158, 11));
        window.set_indicator_text("Already patched".into());
    } else if is_ready {
        window.set_indicator_color(slint::Color::from_rgb_u8(22, 163, 74));
        window.set_indicator_text("Ready to patch".into());
    } else {
        set_not_ready(window);
    }
    if let Some(id) = patch::load_oculus_id(&root) {
        window.set_oculus_id(id.into());
    }
    window.set_status(status.into());
}

fn selected_root(text: &str) -> Result<PathBuf, String> {
    let selected = PathBuf::from(text.trim());
    if selected.join("bin").join("win10").is_dir() {
        return Ok(selected);
    }
    let is_win10 = selected
        .file_name()
        .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("win10"));
    if is_win10
        && selected.is_dir()
        && selected
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("bin"))
    {
        return selected
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .ok_or_else(|| "Could not find the game directory above bin\\win10.".into());
    }
    Err("Choose the Echo VR game directory or its bin\\win10 folder.".into())
}

fn generated_id() -> u64 {
    rand::rng().random_range(1..=9_000_000_000_000_000_000)
}

fn main() -> Result<(), slint::PlatformError> {
    let window = EchoPatcher::new()?;

    // When deployed beside the game executable, no manual path entry is needed.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(root) = patch::root_from_executable(&exe) {
            window.set_game_directory(root.display().to_string().into());
            inspect_installation(&window, root);
        }
    }

    let weak = window.as_weak();
    window.on_browse(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let Some(root) = rfd::FileDialog::new()
            .set_title("Choose the Echo VR game directory")
            .pick_folder()
        else {
            return;
        };
        let selected_text = root.display().to_string();
        window.set_game_directory(selected_text.clone().into());
        match selected_root(&selected_text) {
            Ok(root) => {
                window.set_game_directory(root.display().to_string().into());
                inspect_installation(&window, root);
            }
            Err(error) => {
                set_not_ready(&window);
                window.set_status(error.into());
            }
        }
    });

    let weak = window.as_weak();
    window.on_inspect(move |selected| {
        let Some(window) = weak.upgrade() else { return };
        match selected_root(&selected) {
            Ok(root) => {
                window.set_game_directory(root.display().to_string().into());
                inspect_installation(&window, root)
            }
            Err(error) => {
                set_not_ready(&window);
                window.set_status(error.into());
            }
        }
    });

    let weak = window.as_weak();
    window.on_patch(move || {
        let Some(window) = weak.upgrade() else { return };
        let root = match selected_root(&window.get_game_directory()) {
            Ok(root) => root,
            Err(error) => {
                set_not_ready(&window);
                window.set_status(error.into());
                return;
            }
        };
        let oculus_id = window.get_oculus_id();
        match patch::apply(&root)
            .and_then(|()| patch::write_config(&root, &oculus_id, generated_id()))
        {
            Ok(()) => {
                window.set_patch_enabled(true);
                window.set_indicator_color(slint::Color::from_rgb_u8(245, 158, 11));
                window.set_indicator_text("Already patched".into());
                window.set_status(
                    "Patch complete. libovr-openxr.toml was generated or updated.".into(),
                );
            }
            Err(error) => {
                set_not_ready(&window);
                window.set_status(format!("Patch failed: {error}").into());
            }
        }
    });

    window.run()
}
