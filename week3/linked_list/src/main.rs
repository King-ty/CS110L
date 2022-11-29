use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<u32> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(i);
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    let mut list2 = list.clone();
    list2.pop_front().unwrap();
    println!("origin list: {}", list);
    println!("cloned list (after pop): {}", list2);

    // If you implement iterator trait:
    //for val in &list {
    //    println!("{}", val);
    //}
}
