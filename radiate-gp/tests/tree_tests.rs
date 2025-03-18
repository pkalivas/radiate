#[cfg(test)]
mod test {

    use radiate::*;
    use radiate_gp::*;

    #[test]
    fn test_simpl_tree() {
        let tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::constant(1.0)))
                .attach(TreeNode::new(Op::constant(2.0))),
        );

        assert_eq!(tree.root().map(|node| node.is_valid()), Some(true));
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.size(), 3);
        assert_eq!(tree.eval(&vec![]), 3.0);
    }
}
