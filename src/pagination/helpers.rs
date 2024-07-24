pub(crate) fn extract_scalar<'a>(
    a: &'a dyn liquid::ValueView,
    key: &str,
) -> Option<liquid::model::ScalarCow<'a>> {
    let v = extract_value(a, key)?;
    v.as_scalar()
}

pub(crate) fn extract_tags(value: &dyn liquid::ValueView) -> Option<&dyn liquid::model::ArrayView> {
    let v = extract_value(value, "tags")?;
    v.as_array()
}

pub(crate) fn extract_categories(
    value: &dyn liquid::ValueView,
) -> Option<&dyn liquid::model::ArrayView> {
    let v = extract_value(value, "categories")?;
    v.as_array()
}

fn extract_value<'v>(v: &'v dyn liquid::ValueView, key: &str) -> Option<&'v dyn liquid::ValueView> {
    let o = v.as_object()?;
    o.get(key)
}
