#[macro_export]
macro_rules! print_title {
    ($title:expr) => {
        println!(
            "===================== {} =====================",
            $title.to_uppercase()
        );
    };
}
