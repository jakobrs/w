use w::{order_statistics::OrderStatistics, Tree};

fn main() {
    let mut map = Tree::<i32, i32, OrderStatistics>::new();

    map.insert(3, 2);
    map.insert(1, 5);
    map.insert(6, 3);
    map.insert(3, 4);
    map.insert(0, 10);

    println!("{}", map.root().map_or(0, |node| node.metadata().order));

    for node in map.iter() {
        println!("{} {}", node.key(), node.value());
    }

    println!("map[6] = {}", map[&6]);
}
