fn extract_value<'a>(a: &'a liquid::value::Value, key: &str) -> Option<&'a liquid::value::Value> {
    let key = liquid::value::Scalar::new(key.to_owned());
    a.get(&key)
}

pub fn extract_scalar<'a>(
    a: &'a liquid::value::Value,
    key: &str,
) -> Option<&'a liquid::value::Scalar> {
    extract_value(a, key).and_then(|sort_key| sort_key.as_scalar())
}

pub fn extract_tags(value: &liquid::value::Value) -> Option<&liquid::value::Array> {
    extract_value(value, "tags").and_then(|tags| tags.as_array())
}

pub fn extract_categories(value: &liquid::value::Value) -> Option<&liquid::value::Array> {
    extract_value(value, "categories").and_then(|categories| categories.as_array())
}
