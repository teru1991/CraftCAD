use std::collections::HashMap;
use std::path::Path;

pub fn load_dir(path: &Path) -> Result<HashMap<String, HashMap<String, String>>, String> {
    let ja_path = path.join("ja.json");
    let en_path = path.join("en.json");
    let ja_raw = std::fs::read_to_string(ja_path).map_err(|e| e.to_string())?;
    let en_raw = std::fs::read_to_string(en_path).map_err(|e| e.to_string())?;
    let ja: HashMap<String, String> = serde_json::from_str(&ja_raw).map_err(|e| e.to_string())?;
    let en: HashMap<String, String> = serde_json::from_str(&en_raw).map_err(|e| e.to_string())?;

    for key in ja.keys() {
        if !en.contains_key(key) {
            return Err(format!("missing en translation for key {key}"));
        }
    }
    for key in en.keys() {
        if !ja.contains_key(key) {
            return Err(format!("missing ja translation for key {key}"));
        }
    }

    let mut m = HashMap::new();
    m.insert("ja".to_string(), ja);
    m.insert("en".to_string(), en);
    Ok(m)
}
