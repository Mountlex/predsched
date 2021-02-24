use cc;

fn main() {
    cc::Build::new()
        .file("truncnorm/rtnorm.cpp")
        .include("truncnorm")
        .compile("rtnorm");
}