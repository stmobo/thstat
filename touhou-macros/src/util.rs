use std::iter::once;
use std::mem;

use syn::{Attribute, Expr, Lit, LitStr, Meta};

macro_rules! syn_error_from {
    ($span:expr, $fmt:expr) => {
        syn::Error::new_spanned(&$span, $fmt)
    };
    ($span:expr, $fmt:expr, $($args:tt)*) => {
        syn::Error::new_spanned(&$span, format!($fmt, $($args),*))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharType {
    Uppercase,
    Number,
    Other,
}

impl From<char> for CharType {
    fn from(value: char) -> Self {
        if value.is_numeric() {
            Self::Number
        } else if value.is_uppercase() {
            Self::Uppercase
        } else {
            Self::Other
        }
    }
}

impl Default for CharType {
    fn default() -> Self {
        Self::Other
    }
}

pub fn split_camelcase(s: &str) -> impl Iterator<Item = &'_ str> + '_ {
    use CharType::*;

    let mut iter = s.char_indices().map(|(idx, c)| (idx, CharType::from(c)));
    let (mut prev_idx, mut prev_type) = iter.next().unwrap_or_default();

    iter.filter_map(move |(idx, t)| match (mem::replace(&mut prev_type, t), t) {
        (_, Uppercase) | (Number, Other) | (Other, Number) => Some(idx),
        (Uppercase, _) | (Number, Number) | (Other, Other) => None,
    })
    .map(Some)
    .chain(once(None))
    .filter_map(move |val| match val {
        Some(0) => None,
        Some(idx) => {
            let start = mem::replace(&mut prev_idx, idx);
            Some(&s[start..idx])
        }
        None => Some(&s[prev_idx..]),
    })
}

pub fn camelcase_to_spaced<T: AsRef<str>>(s: T) -> String {
    let mut iter = split_camelcase(s.as_ref());
    iter.next()
        .map(String::from)
        .map(move |first| {
            iter.fold(first, |mut cur, s| {
                cur.push(' ');
                cur.push_str(s);
                cur
            })
        })
        .unwrap_or_default()
}

pub fn attribute_as_lit_str<'a, T>(
    attr_name: &str,
    attrs: T,
) -> Option<Result<&'a LitStr, syn::Error>>
where
    T: IntoIterator<Item = &'a Attribute>,
{
    attrs.into_iter().find_map(|attr| {
        if let Meta::NameValue(kv) = &attr.meta {
            if kv.path.is_ident(attr_name) {
                if let Expr::Lit(lit) = &kv.value {
                    if let Lit::Str(val) = &lit.lit {
                        return Some(Ok(val));
                    }
                } else {
                    return Some(Err(syn_error_from!(
                        kv.value,
                        "expected literal string for attribute {}",
                        attr_name
                    )));
                }
            }
        }

        None
    })
}

pub fn parse_attribute_str<'a, T, U>(attr_name: &str, attrs: T) -> Result<Option<U>, syn::Error>
where
    T: IntoIterator<Item = &'a Attribute>,
    U: syn::parse::Parse,
{
    attribute_as_lit_str(attr_name, attrs)
        .map(|r| r.and_then(|lit| lit.parse()))
        .transpose()
}

pub(super) use syn_error_from;

#[cfg(test)]
mod tests {
    use super::camelcase_to_spaced;

    #[test]
    fn camelcase_basic() {
        assert_eq!(camelcase_to_spaced("ReimuA"), "Reimu A")
    }

    #[test]
    fn camelcase_lowercase() {
        assert_eq!(camelcase_to_spaced("foo"), "foo")
    }

    #[test]
    fn camelcase_start_lowercase() {
        assert_eq!(camelcase_to_spaced("fooBar"), "foo Bar")
    }

    #[test]
    fn camelcase_numbers() {
        assert_eq!(camelcase_to_spaced("4A"), "4 A");
        assert_eq!(camelcase_to_spaced("A4"), "A4");
        assert_eq!(camelcase_to_spaced("Stage4"), "Stage 4");
    }
}
