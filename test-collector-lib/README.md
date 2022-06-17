test-collector introduce custom test runner. You can implement your own `start` and `stop` functions, 
which will be invoked only before and after all tests respectively. 
Also you can override `before_each_test` and `after_each_test` to bring additional cleaning or set_up functionality to each of your tests.
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