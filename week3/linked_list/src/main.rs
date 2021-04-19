use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<char> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);

    for i in 'a'..'z' {
        list.push_front(i);
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    // If you implement iterator trait:
    //for val in &list {
    //    println!("{}", val);
    //}

    let mut new_list = list.clone();
    println!("{}", new_list);
    println!("list size: {}", new_list.get_size());
    println!("top element: {}", new_list.pop_front().unwrap());
    println!("{}", new_list);
    println!("size: {}", new_list.get_size());
    println!("{}", new_list.to_string()); // ToString impl for anything impl Display

}
