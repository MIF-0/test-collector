use std::any::Any;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::time::Instant;
use futures::FutureExt;
use test_collector_utils::IntegrationTestMeta;
use crate::{TestEnvironment, TestResult, TestResults};
use crate::logger::{log_error_static_info, log_error_test, log_static_info, log_test};

pub struct TestRunner<T: TestEnvironment> {
    test_environment: T,
}

impl<T: TestEnvironment> TestRunner<T> {
    pub fn new(test_environment: T) -> TestRunner<T> {
        TestRunner {
            test_environment
        }
    }

    pub fn run_safe(mut self) -> TestResults {
        log_static_info(format_args!("Next step is to start test environment"));
        let spin_up_started_at = Instant::now();
        self.test_environment = self.test_environment.start();
        let start_up_duration = spin_up_started_at.elapsed();
        log_static_info(format_args!("Test environment was started within {:?}", start_up_duration));
        log_static_info(format_args!("***"));

        log_static_info(format_args!("Next step is to run tests"));
        let tests_started_at = Instant::now();
        let (success_tests, failed_tests) = self.run_tests();
        let tests_duration = tests_started_at.elapsed();
        log_static_info(format_args!("All tests finished within {:?}", tests_duration));
        log_static_info(format_args!("***"));

        log_static_info(format_args!("Next step is to stop test environment"));
        let teardown_started_at = Instant::now();
        self.test_environment = self.test_environment.stop();
        let stop_duration = teardown_started_at.elapsed();
        log_static_info(format_args!("Test environment was stopped, within {:?}", stop_duration));
        log_static_info(format_args!("***"));

        let overall_duration = spin_up_started_at.elapsed();
        log_static_info(format_args!("Overall duration {:?}", overall_duration));

        return TestResults {
            success_tests,
            failed_tests,
            start_up_duration,
            tests_duration,
            stop_duration,
        };
    }

    pub fn run(self) {
        let result = self.run_safe();
        let success_test_number = result.success_tests.len();
        let failed_test_number =result.failed_tests.len();
        log_static_info(format_args!("Successful test {}. Failed tests {}",
                                     success_test_number,
                                     failed_test_number,
        ));
        for test in result.success_tests {
            log_static_info(format_args!("Test [{}] ....... PASSED", test.name));
        }
        for test in result.failed_tests {
            log_error_static_info(format_args!("Test [{}] ....... FAILED", test.name));
        }
        if failed_test_number > 0 {
            panic!("Some tests are Failing");
        }
    }

    fn run_tests(&self) -> (Vec<TestResult>, Vec<TestResult>) {
        let number_of_tests = inventory::iter::<IntegrationTestMeta>.into_iter().count();
        log_static_info(format_args!("Found {} tests", number_of_tests));
        let mut successful_tests: Vec<TestResult> = Vec::new();
        let mut failed_tests: Vec<TestResult> = Vec::new();
        for test in inventory::iter::<IntegrationTestMeta> {
            log_test(format_args!("Running Before Each Test for: [{}]", test.name));
            self.test_environment.before_each_test();
            let result = self.run_test(test);
            if result.success {
                successful_tests.push(result);
            } else {
                failed_tests.push(result);
            }
            log_test(format_args!("Running After Each Test for: [{}]", test.name));
            self.test_environment.after_each_test();
        }
        (successful_tests, failed_tests)
    }

    fn run_test(&self, test: &IntegrationTestMeta) -> TestResult {
        let test_started = Instant::now();
        log_test(format_args!("Running Test: [{}]", test.name));
        let result = self.run_test_safe(test);
        let test_duration = test_started.elapsed();
        let success = match result {
            Ok(_) => {
                log_test(format_args!("Test [{}] PASSED. Duration {:?}", test.name, test_duration));
                true
            }
            Err(e) => {
                let error: Result<Box<&'static str>, Box<dyn Any + Send>> = e.downcast();
                if error.is_ok() {
                    let error: Box<&'static str> = error.unwrap();
                    log_error_test(format_args!("Test [{}] finished with ERROR. Duration {:?} \n {:?}",
                                                test.name, test_duration, error));
                } else {
                    log_error_test(format_args!("Test [{}] FAILED. Duration {:?}",
                                                test.name, test_duration));
                }

                false
            }
        };
        TestResult {
            name: test.name.clone(),
            success,
            duration: test_duration,
        }
    }

    fn run_test_safe(&self, test: &IntegrationTestMeta) -> Result<(), Box<dyn Any + Send>> {
        if test.sync_fn.is_some() {
            panic::catch_unwind(test.sync_fn.unwrap())
        } else {
            let async_test = (test.async_fn.as_ref().unwrap())();
            let catch_panic_wrapper = AssertUnwindSafe(async_test).catch_unwind();
            self.test_environment.block_on(catch_panic_wrapper)
        }
    }
}