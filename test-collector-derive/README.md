test-collector-derive will give you '#[collect_test]' which you can use to collect the tests
this lib using `inventory`. 
Supposed to be used together with `test-collector`, but you can use it by its own.

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