extern crate cc;

fn main() {
    cc::Build::new().file("src/mem0.c").compile("mem0");
}