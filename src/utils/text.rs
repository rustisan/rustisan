//! Text utilities for the Rustisan CLI
//!
//! This module provides common text manipulation and formatting utilities.

/// Text utilities
pub struct TextUtils;

impl TextUtils {
    /// Capitalize first letter
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
        }
    }

    /// Convert string to snake_case
    pub fn to_snake_case(input: &str) -> String {
        let mut result = String::new();
        for (i, ch) in input.chars().enumerate() {
            if i > 0 && ch.is_uppercase() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        result
    }

    /// Convert string to PascalCase
    pub fn to_pascal_case(input: &str) -> String {
        input
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
                }
            })
            .collect()
    }

    /// Convert string to camelCase
    pub fn to_camel_case(input: &str) -> String {
        let pascal = Self::to_pascal_case(input);
        let mut chars = pascal.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_lowercase().collect::<String>() + &chars.as_str(),
        }
    }

    /// Convert string to kebab-case
    pub fn to_kebab_case(input: &str) -> String {
        Self::to_snake_case(input).replace('_', "-")
    }

    /// Pluralize a word (simple English rules)
    pub fn pluralize(word: &str) -> String {
        if word.is_empty() {
            return word.to_string();
        }

        let lower = word.to_lowercase();
        if lower.ends_with('s') || lower.ends_with("sh") || lower.ends_with("ch")
           || lower.ends_with('x') || lower.ends_with('z') {
            format!("{}es", word)
        } else if lower.ends_with('y') && !lower.ends_with("ay") && !lower.ends_with("ey")
                 && !lower.ends_with("iy") && !lower.ends_with("oy") && !lower.ends_with("uy") {
            format!("{}ies", &word[..word.len()-1])
        } else if lower.ends_with('f') {
            format!("{}ves", &word[..word.len()-1])
        } else if lower.ends_with("fe") {
            format!("{}ves", &word[..word.len()-2])
        } else {
            format!("{}s", word)
        }
    }

    /// Singularize a word (simple English rules)
    pub fn singularize(word: &str) -> String {
        if word.is_empty() {
            return word.to_string();
        }

        let lower = word.to_lowercase();
        if lower.ends_with("ies") {
            format!("{}y", &word[..word.len()-3])
        } else if lower.ends_with("ves") {
            if word.len() > 4 && &lower[word.len()-4..word.len()-3] == "l" {
                format!("{}f", &word[..word.len()-3])
            } else {
                format!("{}fe", &word[..word.len()-3])
            }
        } else if lower.ends_with("es") && word.len() > 2 {
            let before_es = &lower[word.len()-3..word.len()-2];
            if before_es == "s" || before_es == "x" || before_es == "z" {
                word[..word.len()-2].to_string()
            } else if word.len() > 3 && (&lower[word.len()-4..word.len()-2] == "sh" || &lower[word.len()-4..word.len()-2] == "ch") {
                word[..word.len()-2].to_string()
            } else {
                word[..word.len()-1].to_string()
            }
        } else if lower.ends_with('s') && word.len() > 1 {
            word[..word.len()-1].to_string()
        } else {
            word.to_string()
        }
    }

    /// Truncate text to a specified length with ellipsis
    pub fn truncate(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else if max_length <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &text[..max_length-3])
        }
    }

    /// Pad text to the left with spaces
    pub fn pad_left(text: &str, width: usize) -> String {
        format!("{:>width$}", text, width = width)
    }

    /// Pad text to the right with spaces
    pub fn pad_right(text: &str, width: usize) -> String {
        format!("{:<width$}", text, width = width)
    }

    /// Center text within a specified width
    pub fn center(text: &str, width: usize) -> String {
        format!("{:^width$}", text, width = width)
    }

    /// Remove extra whitespace and normalize spacing
    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// Check if a string is a valid identifier (starts with letter/underscore, contains only alphanumeric/underscore)
    pub fn is_valid_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();
        let first = chars.next().unwrap();

        if !first.is_alphabetic() && first != '_' {
            return false;
        }

        chars.all(|c| c.is_alphanumeric() || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(TextUtils::capitalize("hello"), "Hello");
        assert_eq!(TextUtils::capitalize(""), "");
        assert_eq!(TextUtils::capitalize("a"), "A");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(TextUtils::to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(TextUtils::to_snake_case("hello"), "hello");
        assert_eq!(TextUtils::to_snake_case("HTTPSConnection"), "h_t_t_p_s_connection");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(TextUtils::to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(TextUtils::to_pascal_case("hello"), "Hello");
        assert_eq!(TextUtils::to_pascal_case("user_profile_image"), "UserProfileImage");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(TextUtils::pluralize("cat"), "cats");
        assert_eq!(TextUtils::pluralize("box"), "boxes");
        assert_eq!(TextUtils::pluralize("city"), "cities");
        assert_eq!(TextUtils::pluralize("leaf"), "leaves");
    }

    #[test]
    fn test_singularize() {
        assert_eq!(TextUtils::singularize("cats"), "cat");
        assert_eq!(TextUtils::singularize("boxes"), "box");
        assert_eq!(TextUtils::singularize("cities"), "city");
        assert_eq!(TextUtils::singularize("leaves"), "leave");
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(TextUtils::is_valid_identifier("hello"));
        assert!(TextUtils::is_valid_identifier("_private"));
        assert!(TextUtils::is_valid_identifier("user_name"));
        assert!(TextUtils::is_valid_identifier("User123"));
        assert!(!TextUtils::is_valid_identifier("123user"));
        assert!(!TextUtils::is_valid_identifier("user-name"));
        assert!(!TextUtils::is_valid_identifier(""));
    }
}
