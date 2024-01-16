use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;

pub(crate) trait Node<Id> {
    fn id(&self) -> Id;

    fn format_table(&self) -> String;

    fn parent(&self) -> Option<Id>;

    fn cmp(&self, other: &Self) -> Ordering;
}

#[derive(Debug)]
pub(crate) struct Tree<Id, Node> {
    nodes: HashMap<Id, Node>,
    children: HashMap<Id, Vec<Id>>,
    roots: Vec<Id>,
}

impl<Id, Node> Tree<Id, Node>
where
    Id: Eq + std::hash::Hash + Ord + Clone,
    Node: crate::tree::Node<Id> + Display,
{
    pub(crate) fn new(input: impl Iterator<Item = Node>) -> Self {
        let mut result = Tree {
            nodes: HashMap::new(),
            children: HashMap::new(),
            roots: Vec::new(),
        };
        for node in input {
            if let Some(parent) = node.parent() {
                result
                    .children
                    .entry(parent)
                    .or_insert(Vec::new())
                    .push(node.id());
            } else {
                result.roots.push(node.id());
            }
            result.nodes.insert(node.id(), node);
        }
        let sort_ids = |ids: &mut Vec<Id>| {
            ids.sort_by(|a, b| {
                result
                    .nodes
                    .get(a)
                    .unwrap()
                    .cmp(result.nodes.get(b).unwrap())
            });
        };
        for (_, children) in result.children.iter_mut() {
            sort_ids(children);
        }
        sort_ids(&mut result.roots);
        result
    }

    fn children(&self, id: Id) -> &[Id] {
        self.children.get(&id).map(|x| x.as_slice()).unwrap_or(&[])
    }

    fn get_transitive_parents<'a>(&'a self, node: &Node) -> TransitiveParents<'a, Id, Node> {
        TransitiveParents::new(self, node.parent())
    }

    fn get_transitive_children<'a>(&'a self, node: &Node) -> TransitiveChildren<'a, Id, Node> {
        TransitiveChildren::new(self, node.id())
    }

    pub(crate) fn format<F>(&self, filter: F) -> String
    where
        F: Fn(&Node) -> bool,
    {
        let included = {
            let mut queue: BTreeSet<Id> = self.nodes.keys().cloned().collect();
            let mut result = HashSet::new();
            while let Some(id) = queue.pop_first() {
                let node = self.nodes.get(&id).unwrap();
                if filter(node) {
                    result.insert(node.id());
                    for parent in self.get_transitive_parents(node) {
                        result.insert(parent);
                    }
                    for child in self.get_transitive_children(node) {
                        result.insert(child);
                    }
                }
            }
            result
        };

        let mut acc = "".to_string();
        for root in self.roots.iter() {
            let node = self.nodes.get(root).unwrap();
            self.format_helper(&included, node, true, true, &mut Vec::new(), &mut acc);
        }
        acc.to_string()
    }

    fn format_helper(
        &self,
        included: &HashSet<Id>,
        node: &Node,
        is_root: bool,
        is_last: bool,
        prefixes: &mut Vec<&str>,
        acc: &mut String,
    ) {
        if !included.contains(&node.id()) {
            return;
        }
        *acc += &format!("{} ┃ ", node.format_table());
        for prefix in prefixes.iter() {
            *acc += prefix;
        }
        if !is_root {
            let has_children = !self.children(node.id()).is_empty();
            *acc += if is_last { "└─" } else { "├─" };
            *acc += if has_children { "┬ " } else { "─ " };
        }
        *acc += &format!("{}\n", node);
        let children: Vec<Id> = self
            .children(node.id())
            .iter()
            .filter(|&child| included.contains(child))
            .cloned()
            .collect();
        for (i, child) in children.iter().enumerate() {
            if !is_root {
                prefixes.push(if is_last { "  " } else { "│ " });
            }
            let is_last = i == children.len() - 1;
            self.format_helper(included, &self.nodes[child], false, is_last, prefixes, acc);
            prefixes.pop();
        }
    }
}

struct TransitiveParents<'a, Id, Node>(Option<Id>, &'a Tree<Id, Node>);

impl<'a, Id, Node> TransitiveParents<'a, Id, Node> {
    fn new(tree: &'a Tree<Id, Node>, parent: Option<Id>) -> Self {
        TransitiveParents(parent, tree)
    }
}

impl<'a, Id, Node> Iterator for TransitiveParents<'a, Id, Node>
where
    Id: Eq + Ord + std::hash::Hash + Clone,
    Node: crate::tree::Node<Id> + Display,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.clone() {
            Some(parent) => {
                self.0 = self.1.nodes.get(&parent).unwrap().parent();
                Some(parent.clone())
            }
            None => None,
        }
    }
}

struct TransitiveChildren<'a, Id, Node>(Vec<Id>, &'a Tree<Id, Node>);

impl<'a, Id, Node> TransitiveChildren<'a, Id, Node>
where
    Id: Eq + Ord + std::hash::Hash + Clone,
    Node: crate::tree::Node<Id> + Display,
{
    fn new(tree: &'a Tree<Id, Node>, id: Id) -> Self {
        TransitiveChildren(tree.children(id).to_vec(), tree)
    }
}

impl<'a, Id, Node> Iterator for TransitiveChildren<'a, Id, Node>
where
    Id: Eq + Ord + std::hash::Hash + Clone,
    Node: crate::tree::Node<Id> + Display,
{
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.pop();
        match next {
            Some(child) => {
                for c in self.1.children(child.clone()) {
                    self.0.push(c.clone())
                }
                Some(child)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use unindent::Unindent;

    struct TestNode {
        id: u8,
        parent: Option<u8>,
    }

    impl Display for TestNode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self.id {
                    1 => "one",
                    2 => "two",
                    3 => "three",
                    4 => "four",
                    n => panic!("TestNode out of range: {}", n),
                }
            )
        }
    }

    impl Node<u8> for TestNode {
        fn id(&self) -> u8 {
            self.id
        }

        fn format_table(&self) -> String {
            self.id.to_string()
        }

        fn parent(&self) -> Option<u8> {
            self.parent
        }

        fn cmp(&self, other: &Self) -> Ordering {
            self.id.cmp(&other.id)
        }
    }

    impl TestNode {
        fn new(id: u8, parent: Option<u8>) -> TestNode {
            TestNode { id, parent }
        }
    }

    #[test]
    fn a_single_node_tree() {
        let tree = Tree::new(vec![TestNode::new(1, None)].into_iter());
        assert_eq!(
            tree.format(|_| true),
            "
                1 ┃ one
            "
            .unindent()
        );
    }

    #[test]
    fn b_child() {
        let tree = Tree::new(vec![TestNode::new(1, None), TestNode::new(2, Some(1))].into_iter());
        assert_eq!(
            tree.format(|_| true),
            "
                1 ┃ one
                2 ┃ └── two
            "
            .unindent()
        );
    }

    #[test]
    fn c_children() {
        let tree = Tree::new(
            vec![
                TestNode::new(1, None),
                TestNode::new(2, Some(1)),
                TestNode::new(3, Some(1)),
                TestNode::new(4, Some(1)),
            ]
            .into_iter(),
        );
        assert_eq!(
            tree.format(|_| true),
            "
                1 ┃ one
                2 ┃ ├── two
                3 ┃ ├── three
                4 ┃ └── four
            "
            .unindent()
        );
    }

    #[test]
    fn d_grandchildren() {
        let tree = Tree::new(
            vec![
                TestNode::new(1, None),
                TestNode::new(2, Some(1)),
                TestNode::new(3, Some(2)),
            ]
            .into_iter(),
        );
        assert_eq!(
            tree.format(|_| true),
            "
                1 ┃ one
                2 ┃ └─┬ two
                3 ┃   └── three
            "
            .unindent()
        );
    }

    #[test]
    fn e_bigger() {
        let tree = Tree::new(
            vec![
                TestNode::new(1, None),
                TestNode::new(2, Some(1)),
                TestNode::new(3, Some(2)),
                TestNode::new(4, Some(1)),
            ]
            .into_iter(),
        );
        assert_eq!(
            tree.format(|_| true),
            "
              1 ┃ one
              2 ┃ ├─┬ two
              3 ┃ │ └── three
              4 ┃ └── four
            "
            .unindent()
        );
    }

    #[test]
    fn f_multiple_roots() {
        let tree = Tree::new(vec![TestNode::new(1, None), TestNode::new(2, None)].into_iter());
        assert_eq!(
            tree.format(|_| true),
            "
                1 ┃ one
                2 ┃ two
            "
            .unindent()
        );
    }

    #[test]
    fn g_sorts_roots_by_id() {
        let tree = Tree::new(vec![TestNode::new(2, None), TestNode::new(1, None)].into_iter());
        assert_eq!(
            tree.format(|_| true),
            "
                1 ┃ one
                2 ┃ two
            "
            .unindent()
        );
    }

    mod h_filtering {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn a_filters_nodes() {
            let tree = Tree::new(vec![TestNode::new(1, None), TestNode::new(2, None)].into_iter());
            assert_eq!(
                tree.format(|node| node.id == 2),
                "
                    2 ┃ two
                "
                .unindent()
            );
        }

        #[test]
        fn b_shows_children_of_included_nodes() {
            let tree = Tree::new(
                vec![
                    TestNode::new(1, None),
                    TestNode::new(2, Some(1)),
                    TestNode::new(3, None),
                ]
                .into_iter(),
            );
            assert_eq!(
                tree.format(|node| node.id == 1),
                "
                    1 ┃ one
                    2 ┃ └── two
                "
                .unindent()
            );
        }

        #[test]
        fn c_shows_parents_of_included_nodes() {
            let tree = Tree::new(
                vec![
                    TestNode::new(1, None),
                    TestNode::new(2, Some(1)),
                    TestNode::new(3, None),
                ]
                .into_iter(),
            );
            assert_eq!(
                tree.format(|node| node.id == 2),
                "
                    1 ┃ one
                    2 ┃ └── two
                "
                .unindent()
            );
        }

        #[test]
        fn d_shows_transitive_parents() {
            let tree = Tree::new(
                vec![
                    TestNode::new(1, None),
                    TestNode::new(2, Some(1)),
                    TestNode::new(3, Some(2)),
                ]
                .into_iter(),
            );
            assert_eq!(
                tree.format(|node| node.id == 3),
                "
                    1 ┃ one
                    2 ┃ └─┬ two
                    3 ┃   └── three
                "
                .unindent()
            );
        }

        #[test]
        fn e_bigger() {
            let tree = Tree::new(
                vec![
                    TestNode::new(1, None),
                    TestNode::new(2, Some(1)),
                    TestNode::new(3, Some(2)),
                    TestNode::new(4, None),
                ]
                .into_iter(),
            );
            assert_eq!(
                tree.format(|node| node.id == 2),
                "
                    1 ┃ one
                    2 ┃ └─┬ two
                    3 ┃   └── three
                "
                .unindent()
            );
        }

        #[test]
        fn f_no_unconnected_lines() {
            let tree = Tree::new(
                vec![
                    TestNode::new(1, None),
                    TestNode::new(2, Some(1)),
                    TestNode::new(3, Some(2)),
                    TestNode::new(4, Some(1)),
                ]
                .into_iter(),
            );
            assert_eq!(
                tree.format(|node| node.id == 2),
                "
                    1 ┃ one
                    2 ┃ └─┬ two
                    3 ┃   └── three
                "
                .unindent()
            );
        }
    }
}
