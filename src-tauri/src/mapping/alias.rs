use std::collections::HashMap;
use std::path::Path;

use crate::mapping::normalize_header;

const BUILTIN_JSON: &str = r#"{
  "用户姓名": "姓名",
  "员工姓名": "姓名",
  "人员姓名": "姓名",
  "姓名": "姓名",
  "手机号": "手机号码",
  "联系电话": "手机号码",
  "移动电话": "手机号码",
  "手机": "手机号码",
  "电话": "手机号码",
  "手机号码": "手机号码",
  "身份证": "身份证号",
  "身份证号码": "身份证号",
  "证件号码": "身份证号",
  "证件号": "身份证号",
  "身份证号": "身份证号",
  "部门": "所属部门",
  "部门名称": "所属部门",
  "所在部门": "所属部门",
  "所属单位": "所属单位",
  "组织机构": "所属单位",
  "单位名称": "所属单位"
}"#;

/// Alias lookup table loaded from JSON. Matcher code must not hardcode aliases.
#[derive(Debug, Clone, Default)]
pub struct AliasDictionary {
    canonical_by_alias: HashMap<String, String>,
}

impl AliasDictionary {
    pub fn builtin() -> Self {
        Self::from_json(BUILTIN_JSON).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let raw: HashMap<String, String> = serde_json::from_str(json)?;
        let mut canonical_by_alias = HashMap::new();
        for (alias, canonical) in raw {
            canonical_by_alias.insert(normalize_header(&alias), normalize_header(&canonical));
        }
        Ok(Self { canonical_by_alias })
    }

    pub fn load_or_builtin(path: Option<&Path>) -> Self {
        if let Some(path) = path {
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(dict) = Self::from_json(&text) {
                    return dict;
                }
            }
        }
        Self::builtin()
    }

    pub fn canonical_for(&self, header: &str) -> Option<&str> {
        self.canonical_by_alias
            .get(&normalize_header(header))
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::AliasDictionary;

    #[test]
    fn maps_common_phone_aliases() {
        let dict = AliasDictionary::builtin();
        assert_eq!(dict.canonical_for("联系电话"), Some("手机号码"));
        assert_eq!(dict.canonical_for("手机号"), Some("手机号码"));
        assert_eq!(dict.canonical_for("证件号码"), Some("身份证号"));
        assert_eq!(dict.canonical_for("用户姓名"), Some("姓名"));
    }
}
