use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Display;

pub(crate) trait Node<Id> {
    fn root() -> Id;

    fn id(&self) -> Id;

    fn parent(&self) -> Option<Id>;
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
    Node: crate::tree::Node<Id> + Display + Clone,
{
    pub(crate) fn new(nodes: impl Iterator<Item = Node>) -> Self {
        let mut map = HashMap::new();
        let mut children = HashMap::new();
        let mut roots = Vec::new();
        for node in nodes {
            map.insert(node.id(), node.clone());
            if let Some(parent) = node.parent() {
                children.entry(parent).or_insert(Vec::new()).push(node.id());
            } else {
                roots.push(node.id());
            }
        }
        for (_, children) in children.iter_mut() {
            children.sort();
        }
        roots.sort();
        Tree {
            nodes: map,
            children,
            roots,
        }
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
        Id: Debug,
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
        for prefix in prefixes.iter() {
            *acc += prefix;
        }
        if !is_root {
            let has_children = !self.children(node.id()).is_empty();
            *acc += if is_last { "└─" } else { "├─" };
            *acc += if has_children { "┬ " } else { "─ " };
        }
        *acc += &node.to_string();
        *acc += "\n";
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
    Node: crate::tree::Node<Id> + Display + Clone,
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
    Node: crate::tree::Node<Id> + Display + Clone,
{
    fn new(tree: &'a Tree<Id, Node>, id: Id) -> Self {
        TransitiveChildren(tree.children(id).to_vec(), tree)
    }
}

impl<'a, Id, Node> Iterator for TransitiveChildren<'a, Id, Node>
where
    Id: Eq + Ord + std::hash::Hash + Clone,
    Node: crate::tree::Node<Id> + Display + Clone,
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

    #[derive(Clone)]
    struct TestNode {
        id: u8,
        parent: Option<u8>,
    }

    impl Display for TestNode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.id)
        }
    }

    impl Node<u8> for TestNode {
        fn root() -> u8 {
            1
        }

        fn id(&self) -> u8 {
            self.id
        }

        fn parent(&self) -> Option<u8> {
            self.parent
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
              1
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
              1
              └── 2
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
              1
              ├── 2
              ├── 3
              └── 4
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
              1
              └─┬ 2
                └── 3
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
              1
              ├─┬ 2
              │ └── 3
              └── 4
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
              1
              2
            "
            .unindent()
        );
    }

    #[test]
    fn f_sorts_roots_by_id() {
        let tree = Tree::new(vec![TestNode::new(2, None), TestNode::new(1, None)].into_iter());
        assert_eq!(
            tree.format(|_| true),
            "
              1
              2
            "
            .unindent()
        );
    }

    mod filtering {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn a_filters_nodes() {
            let tree = Tree::new(vec![TestNode::new(1, None), TestNode::new(2, None)].into_iter());
            assert_eq!(
                tree.format(|node| node.id == 2),
                "
                  2
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
                  1
                  └── 2
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
                  1
                  └── 2
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
                  1
                  └─┬ 2
                    └── 3
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
                  1
                  └─┬ 2
                    └── 3
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
                  1
                  └─┬ 2
                    └── 3
                "
                .unindent()
            );
        }
    }
}
