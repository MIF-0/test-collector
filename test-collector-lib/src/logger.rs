use std::fmt::Arguments;
use chrono::Local;
use env_logger::fmt::{Color, Style};
use env_logger::Logger;
use log::{LevelFilter, Record, Log, Level};
use once_cell::sync::OnceCell;
use std::io::Write;
use log::Level::Error;

const LOGGER_TESTING: OnceCell<Logger> = OnceCell::new();
const LOGGER_STATIC_INFO: OnceCell<Logger> = OnceCell::new();


fn create_testing_logger() -> Logger {
    let mut builder = env_logger::Builder::default();

    builder.filter(None, LevelFilter::Info)
        .format(|buf, record| {
            let mut grey_style = buf.style();
            grey_style.set_color(Color::Rgb(128, 128, 128));

            let mut level_style = buf.style();
            level_style.set_color(Color::Blue).set_bold(true);

            let mut test_style = buf.style();
            match record.level() {
                Error => test_style.set_color(Color::Red).set_bold(true),
                _ => test_style.set_color(Color::Green).set_bold(true)
            };

            writeln!(buf, "{}{} {} {} ",
                     grey_style.value("["),
                     Local::now().format("%Y-%m-%dT%H:%M:%S.%s"),
                     level_style.value(record.level()),
                     test_style.value(record.args()))
        });
    builder.build()
}

fn create_test_static_info_logger() -> Logger {
    let mut builder = env_logger::Builder::default();

    builder.filter(None, LevelFilter::Info)
        .format(|buf, record| {
            let mut grey_style = buf.style();
            grey_style.set_color(Color::Rgb(128, 128, 128));

            let mut level_style = buf.style();
            level_style
                .set_color(Color::Blue)
                .set_bold(true);

            let mut test_style = buf.style();
            set_color(&record.level(), &mut test_style);
            test_style.set_bold(true);

            writeln!(buf, "{} {} {} {} {}",
                     grey_style.value("["),
                     grey_style.value("============"),
                     test_style.value(record.args()),
                     grey_style.value("============"),
                     grey_style.value("]"),
            )
        });
    builder.build()
}

fn set_color(log_level: &Level, test_style: &mut Style) {
    match log_level {
        Error => test_style.set_color(Color::Red),
        _ => test_style.set_color(Color::Green),
    };
}

pub fn log_static_info(message: Arguments) {
    let record = Record::builder()
        .args(message)
        .build();
    LOGGER_STATIC_INFO.get_or_init(create_test_static_info_logger).log(&record);
}

pub fn log_error_static_info(message: Arguments) {
    let record = Record::builder()
        .args(message)
        .level(Error)
        .build();
    LOGGER_STATIC_INFO.get_or_init(create_test_static_info_logger).log(&record);
}

pub fn log_test(message: Arguments) {
    let record = Record::builder()
        .args(message)
        .build();
    LOGGER_TESTING.get_or_init(create_testing_logger).log(&record);
}

pub fn log_error_test(message: Arguments) {
    let record = Record::builder()
        .level(Error)
        .args(message)
        .build();
    LOGGER_TESTING.get_or_init(create_testing_logger).log(&record);
}