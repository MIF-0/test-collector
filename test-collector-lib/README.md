test-collector introduce custom test runner.
You will need test-collector-derive, which gives you '#[collect_test]' which you can use to collect the tests
this lib using `inventory`
```rust
    #[collect_test]
    #[test]
    pub fn sync_test_success() {
        println!("Executed sync!");
        assert_eq!(true, true);
    }

    #[collect_test(async)]
    #[actix_web::test]
    pub async fn async_test_success() {
        let client = reqwest::Client::builder()
            .build()
            .expect("error during client build");
        let response = client.get("http://localhost:9090/").send().await;
        assert!(response.is_ok());
    }
```

You will need to create your own main function and in Cargo.toml of you project add this with needed name and path:
```toml
[[test]]
name = "integration"
path = "integration-tests/main.rs"
harness = false
```

example of the main can be found in `test-collector-lib/src/lib.rs:166`