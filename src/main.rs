use w::{
    order_statistics::{OrderStatistics, OsExt, Sequence, SequenceExt},
    Tree,
};

fn main() {
    let mut set = Tree::<i32, (), OrderStatistics>::new();

    set.insert(1, ());
    set.insert(2, ());
    set.insert(3, ());
    set.insert(2, ());
    set.insert(0, ());
    set.insert(10, ());
    set.insert(-3, ());
    set.insert(4, ());

    println!("{:?}", set.contains_key(&2));
    println!("{:?}", set.find_by_rank(5).unwrap().key());

    // println!("{:#?}", set.split_before(&2));

    println!("Hello, world!");

    for node in set.iter() {
        println!("{} {:?}", node.key(), node.value());
    }

    let mut seq = Sequence::<i32>::new();

    seq.insert((), 1);
    seq.insert((), 2);
    seq.push_left(3);
    seq.push_right(4);
    seq.push_right(5);
    seq.insert_at_rank(2, 10);

    for node in seq.iter() {
        println!("{:?} {}", node.key(), node.value());
    }
}
