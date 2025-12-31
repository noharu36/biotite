use std::collections::HashMap;

pub fn parse_front_matter(content: &str) -> (Option<HashMap<String, String>>, &str) {
    if !content.starts_with("---\n") {
        return (None, content);
    }

    if let Some(end_fm_pos) = content.get(4..).and_then(|s| s.find("---\n")) {
        let fm_str = &content[4..4 + end_fm_pos];
        let body_str = &content[4 + end_fm_pos + 4..];

        let mut data = HashMap::new();
        let mut current_list_key: Option<String> = None;

        for l in fm_str.lines() {
            let trimed = l.trim();

            if trimed.starts_with("- ") {
                if let Some(key) = &current_list_key {
                    if let Some(value) = trimed.strip_prefix("- ") {
                        let entry = data.entry(key.to_string()).or_insert(String::new());
                        if !entry.is_empty() {
                            entry.push_str(", ");
                        }
                        entry.push_str(value);
                    }
                }
                continue;
            }

            if let Some((key, value)) = trimed.split_once(":") {
                 if value.is_empty() {
                     current_list_key = Some(key.to_string());
                     data.insert(key.to_string(), String::new());
                 } else {
                     data.insert(key.to_string(), value.trim().to_string());
                     current_list_key = None;
                 }
            }
        }
        return (Some(data), body_str);
    }
    (None, content)
}
