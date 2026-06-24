//! Integration tests exercising the public API of `camelcase`.

use camelcase::{camel_case, camel_case_slice, camel_case_with, Options};

#[test]
fn common_conversions() {
    assert_eq!(camel_case("background-color"), "backgroundColor");
    assert_eq!(camel_case("-webkit-transform"), "webkitTransform");
    assert_eq!(camel_case("_foo_bar"), "_fooBar");
    assert_eq!(camel_case("EnumValue"), "enumValue");
    assert_eq!(camel_case("ALLCAPS"), "allcaps");
}

#[test]
fn pascal_case() {
    let p = Options::new().pascal_case(true);
    assert_eq!(camel_case_with("background-color", p), "BackgroundColor");
    assert_eq!(camel_case_with("foo", p), "Foo");
}

#[test]
fn preserve_consecutive_uppercase() {
    let c = Options::new().preserve_consecutive_uppercase(true);
    assert_eq!(camel_case_with("HTTPRequest", c), "HTTPRequest");
    assert_eq!(camel_case_with("parse-DBURL", c), "parseDBURL");
}

#[test]
fn from_slice() {
    assert_eq!(camel_case_slice(&["unicorn", "rainbow"], Options::default()), "unicornRainbow");
    assert_eq!(camel_case_slice(&[], Options::default()), "");
}
