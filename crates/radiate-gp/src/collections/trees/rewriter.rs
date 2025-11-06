use crate::TreeNode;

pub trait TreeRewriterRule<T> {
    fn apply<'a>(&self, node: &'a mut TreeNode<T>) -> bool;
}

impl<T, F> TreeRewriterRule<T> for F
where
    F: for<'a> Fn(&'a mut TreeNode<T>) -> bool,
{
    fn apply<'a>(&self, node: &'a mut TreeNode<T>) -> bool {
        self(node)
    }
}

pub struct TreeRewriter<T> {
    rules: Vec<Box<dyn TreeRewriterRule<T>>>,
}

impl<T> TreeRewriter<T> {
    pub fn new() -> Self {
        TreeRewriter { rules: vec![] }
    }

    pub fn add_rule<R>(&mut self, rule: R)
    where
        R: TreeRewriterRule<T> + 'static,
    {
        self.rules.push(Box::new(rule));
    }

    pub fn rewrite(&self, node: &mut TreeNode<T>) -> usize {
        let mut rewrites = 0;
        for rule in &self.rules {
            if rule.apply(node) {
                rewrites += 1;
                break;
            }
        }

        if let Some(children) = node.children_mut() {
            for child in children {
                rewrites += self.rewrite(child);
            }
        }

        rewrites
    }
}
