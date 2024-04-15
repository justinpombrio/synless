use rhai::Engine;

pub fn main() {
    let mut engine = Engine::new();
    engine.register_fn("inc", |n: i64| n + 1);

    let ast = engine.compile("inc(2)").unwrap();
    let result = engine.eval_ast::<i64>(&ast).unwrap();
    println!("{}", result);
}
