fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUSTFLAGS", "-C");
    std::env::set_var("target-feature", "-crt-static");
}