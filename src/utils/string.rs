/// 提供 String / &str 的扩展方法
pub trait StringUtilsTrait {
    /// 安全截取前 `max_chars` 个字符（Unicode 安全，不会截断在字符中间）
    ///
    /// 如果不足 `max_chars`，则返回完整字符串。
    fn safe_substring(&self, max_chars: usize) -> &str;
}

impl StringUtilsTrait for str {
    fn safe_substring(&self, max_chars: usize) -> &str {
        let mut end = self.len();
        for (i, (pos, _)) in self.char_indices().enumerate() {
            if i == max_chars {
                end = pos;
                break;
            }
        }
        &self[..end]
    }
}

pub struct StringUtils;
impl StringUtils {
    pub fn mask_sensitive(str: &String) -> String {
        let len = str.len();

        match len {
            0 => String::new(),
            1 => String::from("*"),
            1..=4 => format!("{}{}", &str[0..1], "*".repeat(len - 1)), // 长度0-4直接返回原字符串
            5..=10 => {
                // 长度5-10，保留首尾各2个字符
                format!("{}****{}", &str[0..2], &str[len - 2..])
            }
            _ => {
                // 长度10+，保留首尾各4个字符
                format!("{}****{}", &str[0..4], &str[len - 4..])
            }
        }
    }
}

impl StringUtilsTrait for String {
    fn safe_substring(&self, max_chars: usize) -> &str {
        self.as_str().safe_substring(max_chars)
    }
}
