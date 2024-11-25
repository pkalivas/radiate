use radiate::{Alterer, Chromosome, Crossover, Gene, RandomProvider};

use crate::{Node, Ops, Tree};


pub struct TreeCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub rate: f32,
    pub max_height: i32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TreeCrossover<T>
where
    T: Clone + PartialEq + Default + 'static,
{
    pub fn alterer(rate: f32) -> Alterer<Node<T>, Ops<T>> {
        Alterer::Crossover(Box::new(Self {
            rate,
            max_height: 10,
            _marker: std::marker::PhantomData,
        }))
    }
}

impl<T> Crossover<Node<T>, Ops<T>> for TreeCrossover<T>
where
    T: Clone + PartialEq + Default,
{
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "Tree Crossover"
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut Chromosome<Node<T>, Ops<T>>,
        chrom_two: &mut Chromosome<Node<T>, Ops<T>>,
    ) -> i32 {
        let rate = self.cross_rate();
        let mut cross_count = 0;

        let swap_one_index = RandomProvider::random::<usize>() % chrom_one.len();
        let swap_two_index = RandomProvider::random::<usize>() % chrom_two.len();


        cross_count
    }
}



// using Radiate.Extensions.Evolution.Architects.Interfaces;
// using Radiate.Extensions.Evolution.Architects.NodeCollections;
// using Radiate.Extensions.Evolution.Architects.NodeCollections.Iterators;
// using Radiate.Extensions.Schema;
// using Radiate.Optimizers.Evolution.Alterers;
// using Radiate.Optimizers.Evolution.Genome;
// using Radiate.Randoms;

// namespace Radiate.Extensions.Evolution.Alterers;

// public class TreeCrossover<TNode, TAllele> : Recombinator<TNode>
//     where TNode : class, INodeGene<TNode, TAllele>, new()
// {
//     private readonly int _maxHeight;

//     public TreeCrossover(float crossoverRate = 0.5f, int maxHeight = 10) : base(crossoverRate)
//     {
//         _maxHeight = maxHeight;
//     }
    
//     protected override int Recombine(Population<TNode> population, int[] individuals, long generation)
//     {
//         var random = RandomRegistry.Instance();
        
//         var parentOne = population[individuals[0]].Genotype;
//         var parentTwo = population[individuals[1]].Genotype;
            
//         var oneChromosomeIndex = random.NextInt(parentOne.Length);
//         var twoChromosomeIndex = random.NextInt(parentTwo.Length);
    
//         var oneChromosome = (NodeChromosome<TNode, TAllele>) parentOne.GetChromosome(oneChromosomeIndex);
//         var twoChromosome = (NodeChromosome<TNode, TAllele>) parentTwo.GetChromosome(twoChromosomeIndex);
        
//         var swapOneIndex = random.NextInt(oneChromosome.Length);
//         var swapTwoIndex = random.NextInt(twoChromosome.Length);
        
//         if (!CanCross(oneChromosome, twoChromosome, swapOneIndex, swapTwoIndex))
//         {
//             return 0;
//         }
        
//         var newParentOne = Swap(swapOneIndex, swapTwoIndex, oneChromosome, twoChromosome);
//         var newParentTwo = Swap(swapTwoIndex, swapOneIndex, twoChromosome, oneChromosome);
        
//         if (!newParentOne.IsValid() || !newParentTwo.IsValid())
//         {
//             throw new Exception($"Invalid tree after crossover.\n {newParentOne}\n {newParentTwo}");
//         }
        
//         parentOne.SetChromosome(oneChromosomeIndex, newParentOne);
//         parentTwo.SetChromosome(twoChromosomeIndex, newParentTwo);
        
//         population[individuals[0]] = population[individuals[0]].NewInstance(parentOne, generation);
//         population[individuals[1]] = population[individuals[1]].NewInstance(parentTwo, generation);

//         return 2;
//     }
    
//     private bool CanCross(NodeChromosome<TNode, TAllele> one, NodeChromosome<TNode, TAllele> two, int oneIndex, int twoIndex)
//     {
//         if (one.CollectionType is not CollectionTypes.Tree || two.CollectionType is not CollectionTypes.Tree)
//         {
//             throw new Exception("Chromosomes must be trees to crossover.");
//         }
        
//         if (oneIndex <= 1 || twoIndex <= 1)
//         {
//             return false;
//         }
        
//         var oneDepth = one.Depth(oneIndex);
//         var twoDepth = two.Depth(twoIndex);
        
//         var oneHeight = one.Level(oneIndex);
//         var twoHeight = two.Level(twoIndex);
        
//         return oneHeight + twoDepth <= _maxHeight && twoHeight + oneDepth <= _maxHeight;
//     }
    
//     private static NodeChromosome<TNode, TAllele> Swap(int oneIndex,
//         int twoIndex,
//         NodeChromosome<TNode, TAllele> one,
//         NodeChromosome<TNode, TAllele> two)
//     {
//         var twoSubTree = two.NewInstance(BreadthFirstIterator.Iterate(two, twoIndex).ToArray()).Reindex(one.Length);
        
//         foreach (var node in one[oneIndex].Incoming.Select(val => one[val]))
//         {
//             node.Outgoing.Remove(oneIndex);
//             node.Outgoing.Add(twoSubTree[0].Index);
//             twoSubTree[0].Incoming.Add(node.Index);
//         }
   
//         return one
//             .NewInstance(one
//                 .Except(BreadthFirstIterator.Iterate(one, oneIndex))
//                 .Concat(twoSubTree)
//                 .ToArray())
//             .Reindex();
//     }
// }
