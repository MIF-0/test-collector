//! ## Usage
//! Do not forget to modify Cargo.toml with.
//! If you want you can override before_each_test and after_each_test from the TestEnvironment trait. By default these methods are empty.
//! ```toml
//! [[test]]
//! name = "integration"
//! path = "integration-tests/main.rs"
//! harness = false
//! ```
//!
//! ``` rust
//!    use std::future::Future;
//!    use std::thread;
//!    use actix_web::{App, HttpResponse, HttpServer, Responder};
//!    use actix_web::rt::SystemRunner;
//!    use test_collector_derive::collect_test;
//!    use test_collector::{log_env_info, TestEnvironment};
//!    use test_collector::test_runner::TestRunner;
//!
//!    struct MockTestEnv {
//!     system: SystemRunner,
//!    }
//!
//!    impl TestEnvironment for MockTestEnv {
//!     fn start(self) -> Self {
//!         log_env_info(format_args!("Starting environment"));
//!         thread::spawn(move || {
//!             actix_web::rt::System::new().block_on(async move {
//!                 HttpServer::new(move || App::new()
//!                     .service(hello)
//!                 )
//!                     .bind("127.0.0.1:9090")?
//!                     .run()
//!                     .await
//!             })
//!         });
//!         return self;
//!     }
//!
//!     fn block_on<F: Future>(&self, fut: F) -> F::Output {
//!         self.system.block_on(fut)
//!     }
//!
//!     fn stop(self) -> Self {
//!         log_env_info(format_args!("Here You can stop APP, db or any other services"));
//!         return self;
//!     }
//!    }
//!
//!    #[actix_web::get("/")]
//!    async fn hello() -> impl Responder {
//!     HttpResponse::Ok().body("Hello, world!")
//!    }
//!
//!
//!    #[test]
//!    #[should_panic(expected = "Some tests are Failing")]
//!    fn possible_main() {
//!     let system = actix_web::rt::System::new();
//!     let test_runner = TestRunner::new(MockTestEnv{system});
//!     test_runner.run();
//!    }
//!
//!    #[collect_test]
//!    pub fn sync_test_failing() {
//!     println!("Executed sync!");
//!     assert_eq!(true, false);
//!    }
//!
//!    #[collect_test(async)]
//!    pub async fn async_test_success() {
//!     let client = reqwest::Client::builder()
//!         .build()
//!         .expect("error during client build");
//!     let response = client.get("http://localhost:9090/").send().await;
//!     assert!(response.is_ok());
//!    }
//! ```

pub mod test_runner;
mod logger;

extern crate core;

use std::fmt::Arguments;
use std::future::Future;
use std::time::{Duration};
use crate::logger::log_static_info;

pub trait TestEnvironment {
    fn start(self) -> Self;

    fn before_each_test(&self) {
        // do nothing by default
    }

    fn block_on<F: Future>(&self, fut: F) -> F::Output;

    fn after_each_test(&self) {
        // do nothing by default
    }

    fn stop(self) -> Self;
}

pub struct TestResults {
    pub success_tests: Vec<TestResult>,
    pub failed_tests: Vec<TestResult>,
    pub start_up_duration: Duration,
    pub tests_duration: Duration,
    pub stop_duration: Duration,
}

pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub duration: Duration,
}

pub fn log_env_info(message: Arguments) {
    log_static_info(message);
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;
    use actix_web::{App, HttpResponse, HttpServer, Responder};
    use actix_web::rt::SystemRunner;
    use once_cell::sync::OnceCell;
    use test_collector_derive::collect_test;
    use Ordering::SeqCst;
    use std::rc::Rc;
    use crate::test_runner::TestRunner;
    use crate::{log_env_info, TestEnvironment};

    struct MockTestEnv {
        system: SystemRunner,
        before_each_call: Rc<AtomicU32>,
        after_each_call: Rc<AtomicU32>,
    }

    impl TestEnvironment for MockTestEnv {
        fn start(self) -> Self {
            log_env_info(format_args!("Starting environment"));
            log_env_info(format_args!("Setup of environment Finished"));
            thread::spawn(move || {
                actix_web::rt::System::new().block_on(async move {
                    HttpServer::new(move || App::new()
                        .service(hello)
                    )
                        .bind("127.0.0.1:9090")?
                        .run()
                        .await
                })
            });
            return self;
        }

        fn before_each_test(&self) {
            self.before_each_call.fetch_add(1, SeqCst);
        }

        fn block_on<F: Future>(&self, fut: F) -> F::Output {
            self.system.block_on(fut)
        }

        fn after_each_test(&self) {
            self.after_each_call.fetch_add(1, SeqCst);
        }

        fn stop(self) -> Self {
            log_env_info(format_args!("Teardown started"));
            log_env_info(format_args!("Here You can stop APP, db or any other services"));
            log_env_info(format_args!("Teardown finished"));
            return self;
        }
    }

    #[actix_web::get("/")]
    async fn hello() -> impl Responder {
        HttpResponse::Ok().body("Hello, world!")
    }


    #[test]
    #[should_panic(expected = "Some tests are Failing")]
    fn possible_main() {
        let system = actix_web::rt::System::new();
        let test_runner = TestRunner::new(
            MockTestEnv {
                system,
                before_each_call: Rc::new(AtomicU32::new(0)),
                after_each_call: Rc::new(AtomicU32::new(0)),
            }
        );

        test_runner.run();
    }

    #[test]
    fn check_before_and_after() {
        let system = actix_web::rt::System::new();
        let before_each_call = Rc::new(AtomicU32::new(0));
        let after_each_call = Rc::new(AtomicU32::new(0));
        let test_runner = TestRunner::new(
            MockTestEnv {
                system,
                before_each_call: before_each_call.clone(),
                after_each_call: after_each_call.clone(),
            }
        );

        test_runner.run_safe();
        assert_eq!(before_each_call.fetch_or(0, SeqCst), 4);
        assert_eq!(after_each_call.fetch_or(0, SeqCst), 4);
    }

    #[collect_test]
    pub fn sync_test_failing() {
        println!("Executed sync!");
        assert_eq!(true, false);
    }

    #[collect_test(async)]
    pub async fn async_test_failing() {
        let client = reqwest::Client::builder()
            .build()
            .expect("error during client build");
        let response = client.get("http://localhost:9091/").send().await;
        assert!(response.is_ok());
    }

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
}
