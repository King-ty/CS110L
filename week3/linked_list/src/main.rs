use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<String> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(i.to_string());
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

    assert!(list != list2);
    println!("[{} ] == [{} ] ?: {}", list, list2, list == list2);
    list.pop_front();
    assert!(list == list2);
    println!("[{} ] == [{} ] ?: {}", list, list2, list == list2);
    list.push_front("a".to_string());
    list2.push_front("b".to_string());
    assert!(list != list2);
    println!("[{} ] == [{} ] ?: {}", list, list2, list == list2);

    // If you implement iterator trait:
    //for val in &list {
    //    println!("{}", val);
    //}
}
