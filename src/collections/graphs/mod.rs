//! We consider directed graphs with loops to be our main graph-like structure. Graph can be
//! also extended with edge or vertex labels.
//!
//! ### Evolution operators
//!
//! Compared to most typical representations, we abstract the edge set using a concept of
//! `EvolutionOperator`. This concept is borrowed from the theory of dynamical systems where
//! it usually applies to the evolution of a continuous system.
//!
//! An `EvolutionOperator` is simply a function that accepts a graph `Vertex` and returns
//! an `Iterator` over the successor vertices. The important distinction is that the
//! `EvolutionOperator` does not have to necessarily return "forward" successors, but can
//! also follow the evolution of a graph in the opposite direction.
//!
//! This is useful in several cases:
//!  - For some systems, one cannot construct the actual graph (for example when the vertices
//!  are not known beforehand and are only discovered lazily) — in such case, `EvolutionOperator`
//!  can be still implemented and used by most algorithms.
//!  - Some algorithms (for example many forward-backward SSC decompositions) use sub-routines
//!  that are sometimes performed following the edges forward and sometimes backward. These
//!  sub-routines would have to be implemented twice, or use a algorithm-specific abstraction
//!  over the graph (which would probably look very much like the evolution operator anyway).
//!
//! Also, evolution operators can be wrapped in transformations that enable you to implement
//! things like filtering without caring about the graph itself.
//!
//! TODO: Example!
//!
//! ### Edge and vertex labels
//!
//! Often, graphs contain more than the basic vertex-edge structure. To avoid having a specialized
//! trait for every such variant, we consider `EdgeLabels` and `VertexLabels` that facilitate this
//! extra information.


use std::hash::Hash;
use crate::collections::bitvectors::BitVector58;
use std::collections::HashMap;

/// `EvolutionOperator` is essentially a function $\sigma: A -> 2^B$, i.e. taking an element $s \in A$
/// and returning a subset $t \subseteq B$. For simplicity, the subset is represented as an
/// `Iterator` (because it can be often constructed on-the-fly).
///
/// In most cases, the source and target sets are the same ($A = B$), but this not necessary.
/// For example in edge-labeled graphs, we can have $A$ as the graph vertices and $B$ as
/// pairs (vertex, label).
trait EvolutionOperator {
    type Source;
    type Target;
    type Iterator: Iterator<Item = Self::Target>;

    /// Compute the image of $\sigma(s)$ given a source value $s$.
    fn step(&self, source: Self::Source) -> Self::Iterator;
}

/// A marker trait that should be implemented by all representations of graph vertices.
///
/// `Hash`, `Eq` and `Clone` are fairly reasonable as many algorithms store vertices in `HashMap`,
/// and most need to clone them or test for equality.
///
/// We also require `Copy`, so that we have a unified calling convention (If you have "heavy"
/// vertices, you can implement a caching container - which you should do anyway to reduce
/// memory consumption).
trait Vertex: Clone + Copy + Eq + Hash {}

/// A possible implementation of a `Vertex` is the `BitVector58` which can hold up-to 58
/// boolean values.
impl Vertex for BitVector58 {}

/// An abstract representation of a directed graph with loops.
trait Graph {
    type Vertex: Vertex;
    type Vertices: Iterator<Item = Self::Vertex>;
    type FwdEdges: EvolutionOperator<Source = Self::Vertex, Target = Self::Vertex>;
    type BwdEdges: EvolutionOperator<Source = Self::Vertex, Target = Self::Vertex>;

    /// Returns an iterator over all vertices of this graph.
    fn vertices(&self) -> Self::Vertices;

    /// Returns an `EvolutionOperator` representing the forward edges of this graph.
    fn fwd(&self) -> Self::FwdEdges;

    /// Returns an `EvolutionOperator` representing the backward edges of this graph.
    fn bwd(&self) -> Self::BwdEdges;
}

/// A trait implemented by structures which provide labels for some graph vertices (for example
/// names or metadata supplied by the user).
///
/// A graph can in fact have multiple vertex labellings, so it is usually preferred not to
/// implement `VertexLabelling` by the `Graph` itself. You should preferably make them accessible
/// on demand, similar to how `EvolutionOperators` are created using `fwd` and `bwd`.
///
/// This also allows algorithms to specify that they only require the labeling, not the graph
/// itself.
trait VertexLabels {
    type Label;
    type Vertex: Vertex;
    fn get(&self, vertex: Self::Vertex) -> Self::Label;
}

/// A trait implemented by structures which provide labels for some graph edges (for example
/// parametrisation sets which enable these edges).
///
/// Similar to `VertexLabels`, you usually do not want to implement `EdgeLabels` directly by
/// a `Graph`, but rather provide them as a separate structure.
trait EdgeLabels {
    type Label;
    type Vertex: Vertex;
    fn get(&self, edge: (Self::Vertex, Self::Vertex)) -> Self::Label;
}

/// A simple struct that represents a `Graph` vertex using a `usize` "id".
///
/// This "id" can be often used to access additional data about the vertex, or in general as an
/// index into other data structures (e.g. `VertexLabels`).
struct IdVertex(usize);

// TODO: Implement example explicit graph and vertex storage...
struct HashedVertices<D> {
    storage: HashMap<D, VertexIndex>
}

struct ExplicitGraph<D> {
    hasher: HashedVertices<D>,
    fwd_edges: HashMap<VertexIndex, Vec<VertexIndex>>,
    bwd_edges: HashMap<VertexIndex, Vec<VertexIndex>>
}
