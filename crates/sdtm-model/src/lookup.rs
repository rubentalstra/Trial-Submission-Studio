use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CaseInsensitiveSet {
    map: HashMap<String, String>,
}

impl CaseInsensitiveSet {
    pub fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut map = HashMap::new();
        for name in names {
            let name = name.as_ref();
            let key = name.to_ascii_uppercase();
            map.entry(key).or_insert_with(|| name.to_string());
        }
        Self { map }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.map
            .get(&name.to_ascii_uppercase())
            .map(|value| value.as_str())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(&name.to_ascii_uppercase())
    }
}
