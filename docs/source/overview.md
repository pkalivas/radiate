
<figure>
  <img
    src="../../assets/logo/banner_light.jpeg"
    alt="Radiate"
    style="width:900px; border-radius:20px;"
  />
</figure>

## Key Features

- **Genetic Engine**: The central component orchestrating the genetic algorithm. It manages the ecosystem, evaluates fitness, and handles selection, crossover, and mutation processes. It is designed to be flexible and extensible, allowing customization to fit specific optimization requirements.

- **Codec**: Responsible for encoding and decoding genetic information. It acts as a bridge between the problem space and the solution space, allowing the genetic algorithm to operate on abstract genetic representations while solving real-world problems.

- **Selectors**: Used to choose individuals for reproduction and survival. They play a crucial role in determining the evolutionary pressure applied to the population.

- **Alterers**: Crossover and mutation operators that introduce genetic diversity and enable exploration of the solution space. The library provides a variety of built-in alterers.

- **Fitness Function**: Evaluates how well an individual solves the problem at hand. It is a critical component that guides the evolutionary process by assigning scores to individuals based on their performance.

## Outside Inspirations

Radiate is inspired from a multitude of other genetic algorithm libraries, all of which have their own unique features and capabilities. Some of the most notable inspirations include:

* [carrot](https://github.com/liquidcarrot/carrot): An architecture-free neural network library built around neuroevolution built in javascript.
* [Genevo](https://github.com/innoave/genevo): A Rust library which provides building blocks to run simulations of optimization and search problems using genetic algorithms (GA).
* [Sharpneat](https://github.com/colgreen/sharpneat): A C# library for evolutionary computation, primarily focused on neuroevolution and artificial neural networks.
* [Jenetics](https://jenetics.io): A Genetic Algorithm, Evolutionary Algorithm, Grammatical Evolution, Genetic Programming, and Multi-objective Optimization library, written in modern day Java.

## Research & References

For those interested in diving deeper into the concepts and theories behind genetic algorithms and evolutionary computation, here are some recommended research papers and books that have influenced the development of Radiate:

* [Genetic Algorithms in Search, Optimization, and Machine Learning](https://www.amazon.com/Genetic-Algorithms-Search-Optimization-Machine/dp/0201157675) by David E. Goldberg: A foundational text that covers the principles and applications of genetic algorithms.
* [Genetic Algorithms](https://www.amazon.com/Genetic-Algorithms-Concepts-Textbooks-Processing/dp/1852330724) by K.F. Man, K.S. Tang, and S. Kwong: A comprehensive book that covers the fundamental concepts and techniques of genetic algorithms.

## Guide 

Throughout this user guide, at the end of each section, we will build upon the same example problem to demonstrate how to use the concepts that were just covered.
