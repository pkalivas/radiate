#[cfg(test)]
mod test {

    use radiate::*;
    use radiate_gp::*;

    #[test]
    fn test_simpl_tree() {
        let mut tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::value(1.0)))
                .attach(TreeNode::new(Op::value(2.0))),
        );

        assert!(tree.root().unwrap().is_valid());
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.size(), 3);
        assert_eq!(tree.reduce(&vec![]), 3.0);
    }
}
