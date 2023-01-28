use w::{order_statistics::{OrderStatistics, OsTreeExt}, tree::Node, Tree};

fn main() {
    let mut map = Tree::<i32, i32, OrderStatistics>::new();

    map.insert(3, 2);
    map.insert(1, 5);
    map.insert(6, 3);
    map.insert(3, 4);
    map.insert(0, 10);

    map[&1] = 4;

    println!("{}", map.len());

    for node in map.iter() {
        println!("{} {}", node.key(), node.value());
    }

    println!("map[1] = {}", map[&1]);
    println!("map[#2] = {}", map.find_by_rank(2).unwrap().value());

    let mut set = Tree::<i32, (), OrderStatistics>::new();

    set.insert(3, ());
    set.insert(1, ());
    set.insert(4, ());
    set.insert(4, ());
    set.insert(5, ());

    for node in set.iter() {
        println!("{}", node.key());
    }

    fn upper_bound(node: Option<&Node<i32, (), OrderStatistics>>, key: i32) -> usize {
        node.map_or(0, |node| {
            if node.key() <= &key {
                upper_bound(node.right(), key)
                    + node.left().map_or(0, |left| left.metadata().order)
                    + 1
            } else {
                upper_bound(node.left(), key)
            }
        })
    }
    fn lower_bound(node: Option<&Node<i32, (), OrderStatistics>>, key: i32) -> usize {
        node.map_or(0, |node| {
            if node.key() < &key {
                lower_bound(node.right(), key)
                    + node.left().map_or(0, |left| left.metadata().order)
                    + 1
            } else {
                lower_bound(node.left(), key)
            }
        })
    }

    println!("3: {}", lower_bound(set.root(), 3));
    println!("4: {}", upper_bound(set.root(), 4));
}
