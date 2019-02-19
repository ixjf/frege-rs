mod branch;
mod iter;

#[cfg(feature = "serde_support")]
mod serde_support;

pub use branch::*;
pub use iter::*;

#[cfg(feature = "serde_support")]
pub use serde_support::*;

use id_tree::InsertBehavior::*;
use id_tree::*;
use std::iter::once;

/// The ID of some branch node or some branch in a truth tree. This ID is guaranteed to be unique for the lifetime
/// of the process.
///
/// **Serialization of this struct requires the feature `serde_support` to be enabled.**
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize))]
pub struct TreeId(NodeId);

/// A truth tree generated by the truth tree algorithm.
pub struct TruthTree {
    tree: Tree<Branch>,
}

impl<'a> TruthTree {
    pub(in crate::validity) fn new(main_branch: Branch) -> Self {
        TruthTree {
            tree: TreeBuilder::new().with_root(Node::new(main_branch)).build(),
        }
    }

    /// Returns the ID of the root branch of the tree.
    pub fn main_trunk_id(&self) -> TreeId {
        TreeId(self.tree.root_node_id().unwrap().clone())
    }

    /// Returns an Iterator over the ancestors' ID's from some branch `branch_id`. Includes `branch_id`
    /// as well.
    ///
    /// # Panics
    /// Panics if the ID provided does not represent a branch from this truth tree.
    pub fn traverse_upwards_branch_ids(&'a self, branch_id: &'a TreeId) -> UpwardsBranchesIdsIter {
        UpwardsBranchesIdsIter {
            iter: once(&branch_id.0).chain(
                self.tree
                    .ancestor_ids(&branch_id.0)
                    .expect("invalid branch_id"),
            ),
        }
    }

    /// Returns an Iterator over the ancestor branches of some branch `branch_id`. Includes `branch_id`
    /// as well.
    ///
    /// # Panics
    /// Panics if the ID provided does not represent a branch from this truth tree.
    pub fn traverse_upwards_branches(&'a self, branch_id: &'a TreeId) -> UpwardsBranchesIter {
        UpwardsBranchesIter {
            tree: &self.tree,
            iter: self.traverse_upwards_branch_ids(&branch_id),
        }
    }

    /// Returns an Iterator over the IDs of the descendant branches of some branch `branch_id`. It implements a
    /// pre-order traversal algorithm. It includes `branch_id` as well.
    pub fn traverse_downwards_branches_ids(
        &'a self,
        branch_id: &'a TreeId,
    ) -> DownwardsBranchesIdsIter {
        DownwardsBranchesIdsIter {
            iter: IdsIter {
                tree: &self.tree,
                stack: vec![&branch_id.0],
            },
        }
    }

    /// Returns an Iterator over the descendant branches and their IDs of some branch `branch_id`. It implements
    /// a pre-order traversal algorithm. It includes `branch_id` as well.
    pub fn traverse_downwards_branches(&'a self, branch_id: &'a TreeId) -> DownwardsBranchesIter {
        DownwardsBranchesIter {
            tree: &self.tree,
            iter: self.traverse_downwards_branches_ids(&branch_id),
        }
    }

    /// Returns an Iterator over the direct descendants' IDs of some branch `branch_id`.
    pub fn traverse_branch_direct_descendants_ids(
        &'a self,
        branch_id: &'a TreeId,
    ) -> BranchDirectDescendantsIdsIter {
        BranchDirectDescendantsIdsIter {
            iter: self.tree.children_ids(&branch_id.0).unwrap(),
        }
    }

    /// Returns an Iterator over the direct descendants of some branch `branch_id`.
    pub fn traverse_branch_direct_descendants(
        &'a self,
        branch_id: &'a TreeId,
    ) -> BranchDirectDescendantsIter {
        BranchDirectDescendantsIter {
            tree: &self.tree,
            iter: self.traverse_branch_direct_descendants_ids(&branch_id),
        }
    }

    /// Returns true if the branch has no children, false if it does.
    ///
    /// # Panics
    /// Panics if the ID provided does not represent a branch from this truth tree.
    pub fn branch_is_last_child(&'a self, branch_id: &'a TreeId) -> bool {
        self.tree
            .get(&branch_id.0)
            .expect("invalid branch_id")
            .children()
            .is_empty()
    }

    pub(in crate::validity) fn branch_from_id_mut(&mut self, branch_id: &TreeId) -> &mut Branch {
        self.tree
            .get_mut(&branch_id.0)
            .expect("invalid branch_id")
            .data_mut()
    }

    /// Returns a reference to the `Branch` specified by `branch_id`.
    ///
    /// # Panics
    /// Panics if the ID provided does not represent a branch from this truth tree.
    pub fn branch_from_id(&self, branch_id: &TreeId) -> &Branch {
        self.tree
            .get(&branch_id.0)
            .expect("invalid branch_id")
            .data()
    }

    pub(in crate::validity) fn append_branch_at(
        &mut self,
        branch: Branch,
        as_child_of_branch_id: &TreeId,
    ) -> TreeId {
        assert!(
            !self.branch_from_id(&as_child_of_branch_id).is_closed(),
            "attempt to add child to closed branch"
        );

        TreeId(
            self.tree
                .insert(Node::new(branch), UnderNode(&as_child_of_branch_id.0))
                .expect("invalid branch_id"),
        )
    }

    /// Returns true if there is at least one open branch in the entire tree, false if not.
    pub fn is_open(&self) -> bool {
        self.traverse_downwards_branches_ids(&self.main_trunk_id())
            .filter(|x| self.branch_is_last_child(&x) && !self.branch_from_id(&x).is_closed())
            .count()
            > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{SimpleStatementLetter, Statement, Subscript};

    static BRANCH_NODE_1: BranchNode = BranchNode {
        statement: Statement::Simple(SimpleStatementLetter('A', Subscript(None))),
        derived_from: None,
    };
    static BRANCH_NODE_2: BranchNode = BranchNode {
        statement: Statement::Simple(SimpleStatementLetter('B', Subscript(None))),
        derived_from: None,
    };
    static BRANCH_NODE_3: BranchNode = BranchNode {
        statement: Statement::Simple(SimpleStatementLetter('C', Subscript(None))),
        derived_from: None,
    };

    #[test]
    fn truth_tree_main_trunk_id() {
        let truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        assert_eq!(
            root_id,
            TreeId(truth_tree.tree.root_node_id().unwrap().clone())
        );
    }

    #[test]
    fn truth_tree_traverse_upwards_branch_ids() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let child_branch_1_id =
            truth_tree.append_branch_at(Branch::new(vec![BRANCH_NODE_2.clone()]), &root_id);

        let mut iter = truth_tree.traverse_upwards_branch_ids(&child_branch_1_id);

        assert_eq!(iter.next(), Some(TreeId(child_branch_1_id.0.clone())));
        assert_eq!(iter.next(), Some(truth_tree.main_trunk_id()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn truth_tree_traverse_upwards_branches() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let child_branch_1_id =
            truth_tree.append_branch_at(Branch::new(vec![BRANCH_NODE_2.clone()]), &root_id);

        let mut iter = truth_tree.traverse_upwards_branches(&child_branch_1_id);

        let (_, first_branch) = iter.next().unwrap();

        assert_eq!(
            first_branch
                .statement_from_id(&first_branch.statement_ids().next().unwrap())
                .statement,
            BRANCH_NODE_2.statement
        );

        let (_, second_branch) = iter.next().unwrap();

        assert_eq!(
            second_branch
                .statement_from_id(&second_branch.statement_ids().next().unwrap())
                .statement,
            BRANCH_NODE_1.statement
        );

        match iter.next() {
            Some(_) => assert!(false),
            _ => {}
        }
    }

    #[test]
    fn truth_tree_traverse_downwards_branch_ids() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let child_branch_1_id =
            truth_tree.append_branch_at(Branch::new(vec![BRANCH_NODE_2.clone()]), &root_id);

        let child_branch_2_id = truth_tree
            .append_branch_at(Branch::new(vec![BRANCH_NODE_3.clone()]), &child_branch_1_id);

        let mut iter = truth_tree.traverse_downwards_branches_ids(&root_id);

        assert_eq!(iter.next(), Some(root_id.clone()));
        assert_eq!(iter.next(), Some(child_branch_1_id));
        assert_eq!(iter.next(), Some(child_branch_2_id));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn truth_tree_traverse_branch_direct_descendants_ids() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let child_branch_1_id =
            truth_tree.append_branch_at(Branch::new(vec![BRANCH_NODE_2.clone()]), &root_id);

        truth_tree.append_branch_at(Branch::new(vec![BRANCH_NODE_3.clone()]), &child_branch_1_id);

        let mut iter = truth_tree.traverse_branch_direct_descendants_ids(&root_id);

        assert_eq!(iter.next(), Some(child_branch_1_id));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn truth_tree_branch_is_last_child() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let child_branch_1_id =
            truth_tree.append_branch_at(Branch::new(vec![BRANCH_NODE_2.clone()]), &root_id);

        assert!(
            !truth_tree.branch_is_last_child(&root_id),
            "root is not last child"
        );

        assert!(
            truth_tree.branch_is_last_child(&child_branch_1_id),
            "child_branch_1 is last child"
        );
    }

    #[test]
    fn truth_tree_branch_from_id_mut() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let branch = truth_tree.branch_from_id_mut(&root_id);

        branch.append_statement(BRANCH_NODE_2.clone());

        let mut statements_iter = branch.statement_ids();

        assert_eq!(
            branch
                .statement_from_id(&statements_iter.next().unwrap())
                .statement,
            BRANCH_NODE_1.statement
        );
        assert_eq!(
            branch
                .statement_from_id(&statements_iter.next().unwrap())
                .statement,
            BRANCH_NODE_2.statement
        );
    }

    #[test]
    fn truth_tree_branch_from_id() {
        let truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let branch = truth_tree.branch_from_id(&root_id);

        let first_statement_id = branch.statement_ids().next().unwrap();

        assert_eq!(
            branch.statement_from_id(&first_statement_id).statement,
            BRANCH_NODE_1.statement
        );
    }

    #[test]
    fn truth_tree_append_branch_at() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        let root_id = truth_tree.main_trunk_id();

        let child_branch_1_id =
            truth_tree.append_branch_at(Branch::new(vec![BRANCH_NODE_2.clone()]), &root_id);

        let mut iter = truth_tree.traverse_downwards_branches_ids(&root_id);

        assert_eq!(iter.next(), Some(TreeId(root_id.0.clone())));
        assert_eq!(iter.next(), Some(TreeId(child_branch_1_id.0.clone())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn truth_tree_is_open() {
        let mut truth_tree = TruthTree::new(Branch::new(vec![BRANCH_NODE_1.clone()]));

        assert!(truth_tree.is_open(), "returned not open but tree is open");

        truth_tree
            .branch_from_id_mut(&truth_tree.main_trunk_id())
            .close();

        assert!(!truth_tree.is_open(), "returned open but tree is not open");
    }
}