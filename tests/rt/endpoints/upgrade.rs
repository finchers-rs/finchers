use finchers::endpoints::upgrade::Builder;
use finchers::prelude::*;
use finchers::rt::test;

#[test]
fn test_upgrade() {
    let mut runner = test::runner({
        endpoints::upgrade::builder().map(|builder: Builder| {
            builder
                .header("sec-websocket-accept", "xxxx")
                .finish(|upgraded| {
                    drop(upgraded);
                    Ok(())
                })
        })
    });

    let response = runner.perform("/").unwrap();

    assert!(response.body().is_upgraded());
    assert_eq!(response.status().as_u16(), 101);
    assert_matches!(
        response.headers().get("sec-websocket-accept"),
        Some(h) if h == "xxxx"
    );
}
