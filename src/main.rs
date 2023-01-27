use w::{
    order_statistics::{OrderStatistics, OsNodeExt, OsTreeExt, Sequence, SequenceExt},
    tree::Node,
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

    // println!(
    //     "{:#?}",
    //     w::tree::Node::split_before(set.root_box_mut().take(), &2)
    // );

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
    println!("{:?}", seq.remove_by_rank(3).map(|node| *node.value()));

    for node in seq.iter() {
        println!("{:?} {}", node.key(), node.value());
    }
}
