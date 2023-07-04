# Greedy Minimum-Energy Scheduling

This repository contains an implementation of the greedy algorithm Parallel Left-to-Right described in my [research paper](https://arxiv.org/abs/2307.00949).

## Introduction

The optimization problem addressed is the efficient scheduling of jobs across multiple processors while minimizing energy consumption.
We consider a set of n jobs with individual release times, deadlines, and processing volumes that must be scheduled across m parallel processors. 

Idle processors can be turned off to save energy, but turning them on requires a fixed amount of energy. For the special case of a single processor, the classic greedy algorithm Left-to-Right is a 2-approximation.
The Parallel Left-to-Right algorithm implemented in this repository generalizes this greedy algorithm to multiple processors running in parallel.
You can find more details about this research in the [paper](https://arxiv.org/abs/2307.00949).

## Improvements
- Benchmarking the running time against problem instances drawn from different distributions.
- Benchmarking the energy costs in comparison to the upper bound guaranteed by the approximation factor.
- Use of more efficient algorithm than Edmonds-Karp for maximum flow calculations, e.g. the push-relabel algorithm.

## Contributing

We welcome contributions to this project. If you're interested in contributing, please reach out to me.

## License

This project is licensed under the MIT license. Refer to the [LICENSE](./LICENSE) file for more information.
The data set used for benchmarking is subject to a different license, see next section.

## Dataset Attribution

The dataset used for macro benchmarking in main.rs is provided by Tadumadze, Giorgi; Emde, Simon; Diefenbach, Heiko.
It is available in its original format at this [link](https://zenodo.org/record/3696775).
The dataset is licensed under the [Creative Commons Attribution 4.0 International Public License](https://creativecommons.org/licenses/by/4.0/). 

Modifications have been made to the original dataset for the purposes of this project. Any redistributed version of this dataset, or derivative thereof, is licensed under the same CC BY 4.0 license.

## Contact

If you have any questions or issues, feel free to reach out or create an issue in the [issues](https://github.com/gbid/pltr/issues) section.
