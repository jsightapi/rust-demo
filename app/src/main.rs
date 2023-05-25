mod jsight;

fn main() {
    println!("Hello, world!");
    
    jsight::init().unwrap();
    let stat = jsight::stat().unwrap();
    println!("JSight stat: {}", stat);
}

