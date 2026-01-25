use text_io::read;

#[derive(PartialEq)]
struct Node {
    info: i32,
    next: Option<Box<Node>>,
}

impl Node {
    fn new(data: i32) -> Self {
        Self {
            info: data,
            next: None,
        }
    }
}

struct List {
    head: Option<Box<Node>>,
}

impl List {
    fn new() -> Self {
        Self { head: None }
    }

    fn add_front(&mut self, data: i32) {
        let mut new_node = Node::new(data);
        if self.head == None {
            self.head = Some(Box::new(new_node));
        } else {
            new_node.next = self.head.take();
            self.head = Some(Box::new(new_node));
        }
    }

    fn add_rear(&mut self, data: i32) {
        let new_node = Node::new(data);
        if self.head == None {
            self.head = Some(Box::new(new_node));
        } else {
            let mut next = self.head.as_mut();
            while let Some(node) = next {
                if node.next == None {
                    node.next = Some(Box::new(new_node));
                    break;
                } else {
                    next = node.next.as_mut();
                }
            }
        }
    }

    fn delete_front(&mut self) {
        if let Some(node) = &self.head {
            if node.next == None {
                self.head = None;
                return;
            }
        }
        if self.head == None {
            println!("Nothing to delete");
            return;
        } else {
            let first_node = self.head.take();
            self.head = first_node.unwrap().next.take();
        }
    }

    fn delete_rear(&mut self) {
        if let Some(node) = &self.head {
            if node.next == None {
                self.head = None;
                return;
            }
        }
        if self.head == None {
            println!("Nothing to delete");
            return;
        } else {
            let mut next = &mut self.head;
            while let Some(node) = next {
                if node.next.is_none() {
                    break;
                }

                if node.next.as_ref().unwrap().next.is_none() {
                    node.next = None;
                }

                next = &mut node.next;
            }
        }
    }

    fn display(&self) {
        if self.head == None {
            println!("Empty!!");
        } else {
            let mut ptr = &self.head;

            while let Some(node) = ptr {
                if node.next == None {
                    print!("{}", node.info);
                    break;
                } else {
                    print!("{}->", node.info);
                    ptr = &node.next;
                }
            }
        }
        println!("");
    }
}

fn main() {
    println!("Linked list demo");

    let mut head = List::new();

    loop {
        println!("Option");
        println!(
            "1. insert_front 2. insert_rear\n3. delete_front 4. deleter_rear\n5. Display      6. Exit"
        );
        print!("Enter the choice: ");
        let choice: u8 = read!();
        match choice {
            1 => {
                print!("Enter number: ");
                let data = read!();
                head.add_front(data);
            }
            2 => {
                print!("Enter number: ");
                let data = read!();
                head.add_rear(data);
            }
            3 => head.delete_front(),
            4 => head.delete_rear(),
            5 => head.display(),
            6 => std::process::exit(0),
            _ => continue,
        }

        println!("");
    }
}
