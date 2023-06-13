fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap();
    let content = std::fs::read_to_string(input).expect("should read file");
    let beancount = beancount_exp::parse(&content).expect("failed to parse");
    println!("{}", beancount);
}
