use hashbrown::HashMap;
use korangar_loaders::FileLoader;
use mlua::Lua;
use ragnarok_packets::ItemId;

use super::{HashMapExt, ItemName, ItemResource, Library, Table};
use crate::loaders::GameFileLoader;

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub(super) identified_name: ItemName,
    pub(super) unidentified_name: ItemName,
    pub(super) identified_resource: ItemResource,
    pub(super) unidentified_resource: ItemResource,
    pub(super) identified_description: Vec<String>,
    pub(super) unidentified_description: Vec<String>,
}

fn read_description_lines(item_table: &mlua::Table, field: &str) -> Vec<String> {
    let Ok(table) = item_table.get::<mlua::Table>(field) else {
        return Vec::new();
    };
    table
        .sequence_values::<mlua::String>()
        .flatten()
        .map(|raw| decode_display_text(&raw.as_bytes()))
        .collect()
}

/// Reads a Lua display-name field (item names, descriptions). TC client
/// iteminfo files paste BIG5 Chinese into these fields, so BIG5 takes
/// priority over EUC-KR.
fn read_lua_display_string(item_table: &mlua::Table, field: &str) -> Option<String> {
    let value = item_table.get::<mlua::String>(field).ok()?;
    let bytes = value.as_bytes();
    let decoded = decode_display_text(&bytes);
    if decoded.is_empty() { None } else { Some(decoded) }
}

/// Reads a Lua resource-name field (BMP / sprite filenames). The original RO
/// resource names are Korean, encoded as UTF-8 in modern lubs and EUC-KR in
/// older ones. BIG5 is intentionally last so a BIG5-looking byte sequence
/// doesn't accidentally swallow what should have been an EUC-KR Korean
/// filename.
fn read_lua_resource_string(item_table: &mlua::Table, field: &str) -> Option<String> {
    let value = item_table.get::<mlua::String>(field).ok()?;
    let bytes = value.as_bytes();
    let decoded = decode_resource_name(&bytes);
    if decoded.is_empty() { None } else { Some(decoded) }
}

fn decode_display_text(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    if let Some(text) = encoding_rs::BIG5
        .decode_without_bom_handling_and_without_replacement(bytes)
    {
        return text.into_owned();
    }
    if let Some(text) = encoding_rs::EUC_KR
        .decode_without_bom_handling_and_without_replacement(bytes)
    {
        return text.into_owned();
    }
    String::from_utf8_lossy(bytes).into_owned()
}

fn decode_resource_name(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    if let Some(text) = encoding_rs::EUC_KR
        .decode_without_bom_handling_and_without_replacement(bytes)
    {
        return text.into_owned();
    }
    if let Some(text) = encoding_rs::BIG5
        .decode_without_bom_handling_and_without_replacement(bytes)
    {
        return text.into_owned();
    }
    String::from_utf8_lossy(bytes).into_owned()
}

impl Table for ItemInfo {
    type Key<'a> = ItemId;
    type Storage = HashMap<ItemId, Self>;

    fn load(game_file_loader: &GameFileLoader) -> mlua::Result<Self::Storage> {
        // Modern clients ship iteminfo as system\iteminfo_v2.lub; older clients
        // keep it under data\luafiles514. Try the v2 path first.
        let candidate_paths: &[&str] = &[
            "system\\iteminfo_v2.lub",
            "system\\iteminfo.lub",
            "data\\luafiles514\\lua files\\datainfo\\iteminfo.lub",
        ];
        let path = candidate_paths
            .iter()
            .find(|path| game_file_loader.file_exists(path))
            .copied()
            .unwrap_or(candidate_paths[0]);

        // iteminfo_v2 registers items via `CheckItem(id, data)` rather than a
        // top-level `tbl` literal. Pre-seed a `tbl` and a `CheckItem` stub so
        // either format populates the same global table.
        let data = game_file_loader
            .get(path)
            .unwrap_or_else(|_| panic!("failed to open lua file {}", path));
        let state = Lua::new();
        state
            .load(r#"
                tbl = tbl or {}
                function CheckItem(item_id, item_data)
                    tbl[item_id] = item_data
                    return 1
                end
            "#)
            .exec()?;
        state.load(&data).exec()?;
        // Some scripts wrap their work in a `main` function instead of executing
        // at top level. Invoke it if present (best-effort).
        if let Ok(main) = state.globals().get::<mlua::Function>("main") {
            let _ = main.call::<()>(());
        }

        let globals = state.globals();
        let mut result = HashMap::new();

        if let Ok(table) = globals.get::<mlua::Table>("tbl") {
            // Diagnostic: dump the actual keys present on the first item table.
            if std::env::var("KORANGAR_DUMP_ITEM_CHARS").is_ok() {
                if let Some(Ok((first_id, first_table))) = table.pairs::<u32, mlua::Table>().next() {
                    let mut keys: Vec<String> = Vec::new();
                    for pair in first_table.pairs::<mlua::Value, mlua::Value>() {
                        if let Ok((k, v)) = pair {
                            let key_str = match &k {
                                mlua::Value::String(s) => s.to_str().map(|s| s.to_string()).unwrap_or_else(|_| format!("{:?}", k)),
                                _ => format!("{:?}", k),
                            };
                            let value_preview = match &v {
                                mlua::Value::String(s) => s.to_str().map(|s| s.to_string()).unwrap_or_else(|_| "<binary>".into()),
                                other => format!("{:?}", other),
                            };
                            keys.push(format!("{} = {}", key_str, value_preview));
                        }
                    }
                    eprintln!("[KORANGAR_DUMP_ITEM_CHARS] sample item {} keys:", first_id);
                    for k in &keys {
                        eprintln!("    {}", k);
                    }
                }
            }
            for (item_id, item_table) in table.pairs::<u32, mlua::Table>().flatten() {
                let info = ItemInfo {
                    identified_name: ItemName::from_option(read_lua_display_string(&item_table, "identifiedDisplayName")),
                    unidentified_name: ItemName::from_option(read_lua_display_string(&item_table, "unidentifiedDisplayName")),
                    identified_resource: ItemResource::from_option(read_lua_resource_string(&item_table, "identifiedResourceName")),
                    unidentified_resource: ItemResource::from_option(read_lua_resource_string(&item_table, "unidentifiedResourceName")),
                    identified_description: read_description_lines(&item_table, "identifiedDescriptionName"),
                    unidentified_description: read_description_lines(&item_table, "unidentifiedDescriptionName"),
                };

                result.insert(ItemId(item_id), info);
            }
        }

        // Optional charset dump: set KORANGAR_DUMP_ITEM_CHARS=1 to write every
        // CJK character used across all item names + descriptions to a text
        // file. The font regen tool picks this file up so item info windows
        // get full glyph coverage.
        if std::env::var("KORANGAR_DUMP_ITEM_CHARS").is_ok() {
            eprintln!("[KORANGAR_DUMP_ITEM_CHARS] loaded iteminfo from {}", path);

            // Print a few raw entries so we can diagnose the actual encoding.
            for (item_id, info) in result.iter().take(3) {
                eprintln!(
                    "  item {}: identified_name = {:?} (bytes={:02x?})",
                    item_id.0,
                    info.identified_name.to_string(),
                    info.identified_name.to_string().as_bytes()
                );
                if let Some(desc) = info.identified_description.first() {
                    eprintln!(
                        "    desc[0] = {:?} (bytes={:02x?})",
                        desc,
                        desc.as_bytes()
                    );
                }
            }
            // Every non-ASCII, non-control character that appears in an iteminfo
            // string is a candidate for the font atlas. This covers CJK ideographs,
            // Hangul, kana, full-width forms, CJK punctuation, symbols, arrows,
            // etc. without having to enumerate every Unicode block by hand.
            let is_east_asian = |c: char| (c as u32) > 0x7e && !c.is_control();
            let mut chars: std::collections::BTreeSet<char> = std::collections::BTreeSet::new();
            for info in result.values() {
                for source in [info.identified_name.to_string(), info.unidentified_name.to_string()] {
                    chars.extend(source.chars().filter(|c| is_east_asian(*c)));
                }
                for line in info.identified_description.iter().chain(info.unidentified_description.iter()) {
                    chars.extend(line.chars().filter(|c| is_east_asian(*c)));
                }
            }
            let dump: String = chars.iter().collect();
            let item_count = result.len();
            let char_count = chars.len();
            // Write next to the cargo workspace root so `tools/update-font-charset.ps1`
            // picks it up no matter which subdirectory the binary was launched from.
            let target = std::env::current_dir()
                .ok()
                .and_then(|cwd| {
                    if cwd.join("Cargo.toml").exists() && cwd.join("..").join("Cargo.toml").exists() {
                        cwd.parent().map(|parent| parent.join("iteminfo-charset.txt"))
                    } else {
                        Some(cwd.join("iteminfo-charset.txt"))
                    }
                })
                .unwrap_or_else(|| std::path::PathBuf::from("iteminfo-charset.txt"));
            match std::fs::write(&target, &dump) {
                Ok(_) => eprintln!(
                    "[KORANGAR_DUMP_ITEM_CHARS] wrote {} chars from {} items to {}",
                    char_count,
                    item_count,
                    target.display()
                ),
                Err(error) => eprintln!("[KORANGAR_DUMP_ITEM_CHARS] write failed: {}", error),
            }
        }

        Ok(result.compact())
    }

    fn try_get<'a, 'b>(library: &'a Library, key: Self::Key<'b>) -> Option<&'a Self> {
        library.item_info_table.get(&key)
    }

    fn get<'a, 'b>(library: &'a Library, key: Self::Key<'b>) -> &'a Self {
        static DEFAULT: ItemInfo = ItemInfo {
            identified_name: ItemName::not_found_value(),
            unidentified_name: ItemName::not_found_value(),
            identified_resource: ItemResource::not_found_value(),
            unidentified_resource: ItemResource::not_found_value(),
            identified_description: Vec::new(),
            unidentified_description: Vec::new(),
        };
        Self::try_get(library, key).unwrap_or(&DEFAULT)
    }

}

impl ItemInfo {
    pub fn description<'a>(library: &'a Library, item_id: ItemId, is_identified: bool) -> &'a [String] {
        <Self as Table>::try_get(library, item_id)
            .map(|info| {
                if is_identified {
                    info.identified_description.as_slice()
                } else {
                    info.unidentified_description.as_slice()
                }
            })
            .unwrap_or(&[])
    }
}
