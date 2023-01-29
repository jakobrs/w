use w::{
    order_statistics::{OrderStatistics, OsTreeExt},
    tree::Node,
    Map, Tree,
};

fn main() {
    let mut map = Tree::<i32, i32, OrderStatistics, true>::new();

    map.insert(1, 3);
    map.insert(2, 4);
    map.insert(-1, 5);
    map.insert(1, 6);
    map.insert(7, 8);
    map[&7] = 2;

    for node in map.iter() {
        println!(
            "map[{key}] = {value}",
            key = node.key(),
            value = node.value()
        );
    }

    {
        let node = map.find_by_rank(3).unwrap();
        println!(
            "map[#3] = map[{key}] = {value}",
            key = node.key(),
            value = node.value()
        );
    }

    map.remove_by_rank(0);
    for node in map.iter() {
        println!(
            "map[{key}] = {value}",
            key = node.key(),
            value = node.value()
        );
    }

    fn upper_bound(node: Option<&Node<i32, i32, OrderStatistics, true>>, key: i32) -> usize {
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
    fn lower_bound(node: Option<&Node<i32, i32, OrderStatistics, true>>, key: i32) -> usize {
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

    println!(
        "{}",
        upper_bound(map.root(), 7) - lower_bound(map.root(), 2)
    );

    let mut other_map: Map<i32, i32> = [(1, 2), (3, 2), (-1, 7), (0, 4)].into_iter().collect();

    println!("Removed pair: {:?}", other_map.remove_key(&3));
    println!("Removed pair: {:?}", other_map.remove_key(&3));

    println!(
        "{:?}",
        other_map
            .iter()
            .map(|node| (node.key(), node.value()))
            .collect::<Vec<_>>()
    );
}
