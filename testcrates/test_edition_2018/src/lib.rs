#[cfg(test)]
mod tests {
    use finchers::path;
    use finchers::routes;
    use finchers::prelude::*;
    use finchers::endpoint::syntax;

    #[test]
    fn test_path_macro() {
        let _ = path!(@get /);
        let _ = path!(@get / "foo" / u32);
        let _ = path!(@get / "foo" / { syntax::remains::<String>() });
    }

    #[test]
    fn test_routes_macro() {
        let _ = routes![
            endpoint::unit(),
            endpoint::cloned(42),
        ];
    }
}
