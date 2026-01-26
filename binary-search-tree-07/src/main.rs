use text_io::read;

#[derive(PartialEq)]
struct Node {
    info: i32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(info: i32) -> Self {
        Self {
            info,
            left: None,
            right: None,
        }
    }
}

struct BST {
    root: Option<Box<Node>>,
}

impl BST {
    fn new() -> Self {
        Self { root: None }
    }

    fn insert(&mut self, node: Node) {
        if self.root == None {
            self.root = Some(Box::new(node));
            return;
        }
        let cur = self.root.as_mut();
        let cur = cur.unwrap();

        if cur.left == None && cur.right == None {
            if node.info < cur.info {
                cur.left = Some(Box::new(node));
            } else {
                cur.right = Some(Box::new(node));
            }
            return;
        }

        let mut cur = self.root.as_mut();

        while let Some(current_node) = cur {
            if node.info < current_node.info {
                if current_node.left == None {
                    current_node.left = Some(Box::new(node));
                    break;
                }
                cur = current_node.left.as_mut();
            } else {
                if current_node.right == None {
                    current_node.right = Some(Box::new(node));
                    break;
                }
                cur = current_node.right.as_mut();
            }
        }
    }

    fn contains(&self, info: i32) {
        let mut cur = self.root.as_ref();
        while let Some(current_node) = cur {
            if info == current_node.info {
                println!("BST Contains: {}", info);
                return;
            }

            if info < current_node.info {
                cur = current_node.left.as_ref();
            } else {
                cur = current_node.right.as_ref();
            }
        }
        println!("{} is not in bst", info);
    }

    fn preorder(first: Option<&Box<Node>>) {
        if let Some(cur) = first {
            print!("{} ", cur.info);
            Self::preorder(cur.left.as_ref());
            Self::preorder(cur.right.as_ref());
        }
    }

    fn inorder(first: Option<&Box<Node>>) {
        if let Some(cur) = first {
            Self::inorder(cur.left.as_ref());
            print!("{} ", cur.info);
            Self::inorder(cur.right.as_ref());
        }
    }

    fn postorder(first: Option<&Box<Node>>) {
        if let Some(cur) = first {
            Self::postorder(cur.left.as_ref());
            Self::postorder(cur.right.as_ref());
            print!("{} ", cur.info);
        }
    }
}

fn main() {
    println!("Demo BST Program");

    let mut bst = BST::new();

    loop {
        println!("1. insert 2. search 3. display 4. exit");
        print!("Enter the choice: ");
        let choice: u8 = read!();

        match choice {
            1 => {
                print!("Enter the number: ");
                let num: i32 = read!();
                let node = Node::new(num);
                bst.insert(node);
            }
            2 => {
                print!("Enter the number to search: ");
                let num: i32 = read!();
                bst.contains(num);
            }
            3 => {
                print!("Preorder -> ");
                BST::preorder(bst.root.as_ref());
                print!("\nInorder -> ");
                BST::inorder(bst.root.as_ref());
                print!("\nPostorder -> ");
                BST::postorder(bst.root.as_ref());
                println!("");
            }
            4 => {
                std::process::exit(0);
            }
            _ => continue,
        }
    }
}
