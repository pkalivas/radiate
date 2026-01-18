!!! warning ":construction: Under Construction :construction:"

    As of `01/15/2026`: These docs are a work in progress. Please check back later for updates.

    Probabilistic Graphical Models (PGMs) are currently being developed and integrated into radiate. This documentation will be updated to reflect the latest features and capabilities as they become available.

    As of the next release of radiate (1.2.21) the current (1.2.20) state of PGMs will be deprecated and removed. ie: the current PGM implementation will be replaced with a more robust and feature-complete version in the upcoming release.


Probabilistic Graphical Models (PGMs) in radiate allow for the representation and manipulation of probabilistic relationships between variables. They are a way to represent, reason about, and compute with uncertainty by combining probability theory and graph theory. They provide a structured framework for modeling complex systems where variables are interdependent and uncertainty is inherent.

Suppose you have many random variables that is exponentially large and infeasible to store or reason about directly. For example, representing a full joint distribution such as

$$
P(X_1, X_2, \dots, X_n)
$$

is often impractical because the number of possible configurations grows exponentially with the number of variables.

PGMs exploit conditional independence:

- If variable A only depends on B and C, you don’t need to model how it depends on everything else.

This allows us to factor the joint distribution into smaller, more manageable pieces. For example, if we have a set of variables {X1, X2, X3} where X1 depends on X2 and X3, we can express the joint distribution as:

$$
P(X_1, X_2, X_3) = P(X_1 | X_2, X_3) * P(X_2) * P(X_3)
$$

So, in english, you could say "if its raining (X2) and its 8am (X3), then the probability of me being late for work (X1) is high".

This factorization reduces the complexity of the model and makes it easier to work with.
