pub fn str_coalesce<'a>(opt: &'a Option<String>, fallback: &'a str) -> &'a str {
    opt.as_deref().unwrap_or(fallback)
}
