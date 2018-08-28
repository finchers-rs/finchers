extern crate version_check;

fn main() {
    match version_check::is_nightly() {
        Some(true) => {}
        _ => panic!("requires nightly compiler"),
    }

    match version_check::is_min_date("2018-08-26") {
        Some((true, _)) => {}
        _ => panic!("requires the compiler released on or after 2018-08-26"),
    }
}
