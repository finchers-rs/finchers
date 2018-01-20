use egg_mode::search::ResultType;

#[derive(Debug, Deserialize)]
pub struct SearchTwitterParam {
    #[serde(rename = "q")] pub query: String,
    #[serde(deserialize_with = "parse::result_type")] pub result_type: Option<ResultType>,
    pub count: Option<u32>,
}

mod parse {
    extern crate serde;
    use self::serde::de::{self, Deserializer, Visitor};
    use super::ResultType;
    use std::fmt;

    pub fn result_type<'de, D>(de: D) -> Result<Option<ResultType>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ResultTypeVisitor;
        impl<'de> Visitor<'de> for ResultTypeVisitor {
            type Value = Option<ResultType>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a result type ('recent', 'popular' or 'mixed')")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, de: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                de.deserialize_str(self)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "recent" => Ok(Some(ResultType::Recent)),
                    "popular" => Ok(Some(ResultType::Popular)),
                    "mixed" => Ok(Some(ResultType::Mixed)),
                    s => Err(E::custom(format!("`{}' is not a valid result type", s))),
                }
            }
        }
        de.deserialize_option(ResultTypeVisitor)
    }
}

// TODO: use impl Trait
#[macro_export]
macro_rules! build_endpoint {
    () => {{
        use endpoint::SearchTwitterParam;
        use error::SearchTwitterError;
        use finchers::endpoint::prelude::*;
        use finchers::contrib::urlencoded::serde::{queries, Form};

        endpoint("search").with(choice![
            get(queries()),
            post(body()).map(|Form(queries)| queries)
        ])
        .assert_types::<SearchTwitterParam, SearchTwitterError>()
    }};
}
