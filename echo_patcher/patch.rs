//! Verified Echo VR game-local patching and configuration generation.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const ECHO_RVA: u32 = 0x1365_B21;
const ECHO_ORIGINAL: [u8; 4] = [0x85, 0xc0, 0x75, 0x11];
const ECHO_PATCHED: [u8; 4] = [0x90, 0x90, 0xeb, 0x11];
const PNS_RVA: u32 = 0x98_B5A;
const PNS_ORIGINAL: [u8; 2] = [0x74, 0x27];
const PNS_PATCHED: [u8; 2] = [0xeb, 0x27];

pub const ECHO_ORIGINAL_SHA256: &str =
    "b6d08277e5846900c81004b64b298df6acba834b69700a640b758bda94a52043";
pub const PNS_ORIGINAL_SHA256: &str =
    "eab10108e45500eaf9378a8627b18218db6c29103775520836f57c9d9a4c755c";
const ECHO_PATCHED_SHA256: &str =
    "2b463fe2f73dc7cb6c4b6ceeb367bd375958c4d7de176b58b6c9c3be57597596";
const PNS_PATCHED_SHA256: &str = "c56fe02e01240d869583fa86b701c004239b0b09683a7ab12dae0f6c60576726";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileState {
    Missing,
    Original,
    Patched,
    Unsupported,
}

impl FileState {
    pub fn text(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Original => "original (ready to patch)",
            Self::Patched => "patched (can safely re-patch)",
            Self::Unsupported => "unsupported or modified",
        }
    }
}

#[derive(Debug)]
pub struct FileInspection {
    pub path: PathBuf,
    pub state: FileState,
    pub sha256: Option<String>,
}

#[derive(Debug)]
pub struct Inspection {
    pub root: PathBuf,
    pub echo: FileInspection,
    pub pns: FileInspection,
    pub echo_original: FileInspection,
    pub pns_original: FileInspection,
}

fn win10(root: &Path) -> PathBuf {
    root.join("bin").join("win10")
}

pub fn root_from_executable(path: &Path) -> Option<PathBuf> {
    let win10 = path.parent()?;
    if !win10
        .file_name()?
        .to_string_lossy()
        .eq_ignore_ascii_case("win10")
    {
        return None;
    }
    win10.parent()?.parent().map(Path::to_path_buf)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn rva_offset(image: &[u8], rva: u32) -> Result<usize, String> {
    if image.get(0..2) != Some(b"MZ") {
        return Err("not a DOS/PE file".into());
    }
    let pe_offset = read_u32(image, 0x3c)? as usize;
    if image.get(pe_offset..pe_offset + 4) != Some(b"PE\0\0") {
        return Err("invalid PE signature".into());
    }
    let sections = read_u16(image, pe_offset + 6)? as usize;
    let optional_size = read_u16(image, pe_offset + 20)? as usize;
    let section_start = pe_offset + 24 + optional_size;
    for index in 0..sections {
        let offset = section_start + index * 40;
        let virtual_size = read_u32(image, offset + 8)?;
        let virtual_address = read_u32(image, offset + 12)?;
        let raw_size = read_u32(image, offset + 16)?;
        let raw_offset = read_u32(image, offset + 20)?;
        if rva >= virtual_address
            && rva < virtual_address.saturating_add(virtual_size.max(raw_size))
        {
            return Ok((raw_offset + rva - virtual_address) as usize);
        }
    }
    Err(format!("RVA {rva:#x} is not in a PE section"))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let value = bytes.get(offset..offset + 2).ok_or("truncated PE header")?;
    Ok(u16::from_le_bytes(
        value.try_into().expect("length checked"),
    ))
}
fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = bytes.get(offset..offset + 4).ok_or("truncated PE header")?;
    Ok(u32::from_le_bytes(
        value.try_into().expect("length checked"),
    ))
}

fn state(bytes: &[u8], rva: u32, original: &[u8], patched: &[u8]) -> FileState {
    let Ok(offset) = rva_offset(bytes, rva) else {
        return FileState::Unsupported;
    };
    match bytes.get(offset..offset + original.len()) {
        Some(value) if value == original => FileState::Original,
        Some(value) if value == patched => FileState::Patched,
        _ => FileState::Unsupported,
    }
}

fn inspect_file(
    path: PathBuf,
    rva: u32,
    original: &[u8],
    patched: &[u8],
    original_hash: &str,
    patched_hash: &str,
) -> FileInspection {
    match fs::read(&path) {
        Ok(bytes) => {
            let hash = sha256(&bytes);
            let mut file_state = state(&bytes, rva, original, patched);
            if (file_state == FileState::Original && hash != original_hash)
                || (file_state == FileState::Patched && hash != patched_hash)
            {
                file_state = FileState::Unsupported;
            }
            FileInspection {
                path,
                state: file_state,
                sha256: Some(hash),
            }
        }
        Err(_) => FileInspection {
            path,
            state: FileState::Missing,
            sha256: None,
        },
    }
}

pub fn inspect(root: &Path) -> Inspection {
    let dir = win10(root);
    Inspection {
        root: root.to_path_buf(),
        echo: inspect_file(
            dir.join("echovr.exe"),
            ECHO_RVA,
            &ECHO_ORIGINAL,
            &ECHO_PATCHED,
            ECHO_ORIGINAL_SHA256,
            ECHO_PATCHED_SHA256,
        ),
        pns: inspect_file(
            dir.join("pnsovr.dll"),
            PNS_RVA,
            &PNS_ORIGINAL,
            &PNS_PATCHED,
            PNS_ORIGINAL_SHA256,
            PNS_PATCHED_SHA256,
        ),
        echo_original: inspect_file(
            dir.join("echovr.exe.original"),
            ECHO_RVA,
            &ECHO_ORIGINAL,
            &ECHO_PATCHED,
            ECHO_ORIGINAL_SHA256,
            ECHO_PATCHED_SHA256,
        ),
        pns_original: inspect_file(
            dir.join("pnsovr.dll.original"),
            PNS_RVA,
            &PNS_ORIGINAL,
            &PNS_PATCHED,
            PNS_ORIGINAL_SHA256,
            PNS_PATCHED_SHA256,
        ),
    }
}

fn source_for_patch(live: &FileInspection, original: &FileInspection) -> Result<PathBuf, String> {
    match original.state {
        FileState::Original => Ok(original.path.clone()),
        FileState::Missing if live.state == FileState::Original => Ok(live.path.clone()),
        FileState::Missing => Err(format!(
            "{} is missing; cannot safely recover an already-patched or unsupported file",
            original.path.display()
        )),
        _ => Err(format!(
            "{} is not a supported original",
            original.path.display()
        )),
    }
}

fn patch_bytes(
    mut bytes: Vec<u8>,
    rva: u32,
    original: &[u8],
    replacement: &[u8],
) -> Result<Vec<u8>, String> {
    let offset = rva_offset(&bytes, rva)?;
    if bytes.get(offset..offset + original.len()) != Some(original) {
        return Err(format!("unexpected bytes at RVA {rva:#x}"));
    }
    bytes[offset..offset + replacement.len()].copy_from_slice(replacement);
    Ok(bytes)
}

fn write_replacing(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("libovr-openxr.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| format!("write {}: {error}", temporary.display()))?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "replace {}: {error} (is the game running?)",
            path.display()
        ));
    }
    Ok(())
}

/// Reconstruct both live files from verified originals, then apply both patches.
pub fn apply(root: &Path) -> Result<(), String> {
    let checked = inspect(root);
    let echo_source = source_for_patch(&checked.echo, &checked.echo_original)?;
    let pns_source = source_for_patch(&checked.pns, &checked.pns_original)?;
    let echo = fs::read(&echo_source)
        .map_err(|error| format!("read {}: {error}", echo_source.display()))?;
    let pns =
        fs::read(&pns_source).map_err(|error| format!("read {}: {error}", pns_source.display()))?;
    let echo = patch_bytes(echo, ECHO_RVA, &ECHO_ORIGINAL, &ECHO_PATCHED)?;
    let pns = patch_bytes(pns, PNS_RVA, &PNS_ORIGINAL, &PNS_PATCHED)?;
    let dir = win10(root);
    if checked.echo_original.state == FileState::Missing {
        fs::copy(&checked.echo.path, &checked.echo_original.path)
            .map_err(|error| format!("create Echo backup: {error}"))?;
    }
    if checked.pns_original.state == FileState::Missing {
        fs::copy(&checked.pns.path, &checked.pns_original.path)
            .map_err(|error| format!("create PNS backup: {error}"))?;
    }
    write_replacing(&dir.join("echovr.exe"), &echo)?;
    write_replacing(&dir.join("pnsovr.dll"), &pns)?;
    Ok(())
}

fn toml_string(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.contains(['"', '\\', '\n', '\r']) {
        return Err(
            "Oculus ID must be non-empty and cannot contain quotes, backslashes, or newlines"
                .into(),
        );
    }
    Ok(value.to_owned())
}

pub fn load_oculus_id(root: &Path) -> Option<String> {
    let text = fs::read_to_string(win10(root).join("libovr-openxr.toml")).ok()?;
    let config: toml::Value = text.parse().ok()?;
    config
        .get("user")?
        .get("oculus_id")?
        .as_str()
        .map(str::to_owned)
}

pub fn write_config(root: &Path, oculus_id: &str, generated_id: u64) -> Result<(), String> {
    let oculus_id = toml_string(oculus_id)?;
    let output = win10(root).join("libovr-openxr.toml");
    let mut config = fs::read_to_string(&output)
        .ok()
        .and_then(|text| text.parse::<toml::Value>().ok())
        .unwrap_or_else(|| toml::Value::Table(Default::default()));
    let Some(table) = config.as_table_mut() else {
        return Err(format!("{} is not a TOML table", output.display()));
    };
    let user = table
        .entry("user")
        .or_insert_with(|| toml::Value::Table(Default::default()));
    let Some(user) = user.as_table_mut() else {
        return Err(format!(
            "{} contains a non-table [user] value",
            output.display()
        ));
    };
    let id = user
        .get("id")
        .and_then(toml::Value::as_integer)
        .filter(|id| *id > 0)
        .map(|id| id as u64)
        .unwrap_or(generated_id);
    let org_id = user
        .get("org_id")
        .and_then(toml::Value::as_integer)
        .filter(|id| *id > 0)
        .map(|id| id as u64)
        .unwrap_or(id);
    user.insert("id".into(), toml::Value::Integer(id as i64));
    user.insert("org_id".into(), toml::Value::Integer(org_id as i64));
    user.insert("oculus_id".into(), toml::Value::String(oculus_id));
    let text = toml::to_string_pretty(&config)
        .map_err(|error| format!("serialize {}: {error}", output.display()))?;
    fs::write(
        &output,
        format!("# Generated by libovr-openxr. Keep these values stable after linking.\n\n{text}"),
    )
    .map_err(|error| format!("write {}: {error}", output.display()))
}
