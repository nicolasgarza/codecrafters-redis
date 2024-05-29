pub fn make_bulk_string(words: Vec<&str>) -> String {
    let mut res = String::new();
    for word in words {
        res.push_str(&format!("\r\n{}", word));
    }
    format!("${}{}\r\n", res.len() - 2, res)
}


pub fn make_simple_string(s: &str) -> String {
    format!("+{}\r\n", s)
}

pub fn make_null_bulk_string() -> String {
    "$-1\r\n".to_string()
}

pub fn make_resp_array(words: Vec<&str>) -> String {
    let mut res = format!("*{}\r\n", words.len());
    let items = words.clone();
    let items = items.iter().map(|item| format!("${}\r\n{}\r\n", item.len(), item)).collect::<String>();
    res.push_str(&items);
    res
}

pub fn get_words(s: String) -> Vec<String> {
    s.split("\r\n")
        .filter_map(|part| {
            if part.starts_with("$") || part.starts_with("*") || part.is_empty() {
                None
            } else {
                Some(part.to_string())
            }
        })
        .collect()
}

pub mod servers;