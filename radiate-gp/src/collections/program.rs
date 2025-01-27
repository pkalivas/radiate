use super::{Eval, EvalMut, Graph, GraphEvaluator, Tree};

pub enum Regressor<T> {
    Tree(Tree<T>),
    Forest(Vec<Tree<T>>),
    Graph(Graph<T>),
}

impl<T> EvalMut<[T], Vec<T>> for Regressor<T>
where
    T: Clone + Default,
{
    fn eval_mut(&mut self, input: &[T]) -> Vec<T> {
        match self {
            Regressor::Tree(tree) => vec![tree.eval(input)],
            Regressor::Forest(forest) => forest.iter().map(|tree| tree.eval(input)).collect(),
            Regressor::Graph(graph) => {
                let mut evaluator = GraphEvaluator::new(graph);
                evaluator.eval_mut(input)
            }
        }
    }
}

impl<T> EvalMut<Vec<Vec<T>>, Vec<Vec<T>>> for Regressor<T>
where
    T: Clone + Default,
{
    fn eval_mut(&mut self, input: &Vec<Vec<T>>) -> Vec<Vec<T>> {
        match self {
            Regressor::Tree(tree) => input
                .into_iter()
                .map(|input| vec![tree.eval(&input)])
                .collect(),
            Regressor::Forest(forest) => input
                .into_iter()
                .map(|input| forest.iter().map(|tree| tree.eval(&input)).collect())
                .collect(),
            Regressor::Graph(graph) => {
                let mut evaluator = GraphEvaluator::new(graph);
                input
                    .into_iter()
                    .map(|input| evaluator.eval_mut(&input))
                    .collect()
            }
        }
    }
}
