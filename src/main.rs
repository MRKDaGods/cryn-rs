// Hide console in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod platform;

fn main() {
    //cryn_rs::services::course_manager::test_courses();

    platform::run();
}
