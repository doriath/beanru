fn main() {
    let content = std::fs::read_to_string(std::env::args().skip(1).next().unwrap()).expect("should read file");
    let beancount = beancount_exp::parse(&content).expect("failed to parse");
    println!("{:?}", beancount);
}
