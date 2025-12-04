use nannou::prelude::*;

struct Graph<N, E, G>
    where N: Clone + Default,
          E: Clone + Default,
          G: Clone
{
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
    inner: G,
}

#[derive(Clone, Default)]
struct Node<T>
    where T: Clone + Default
{
    inner: T,
}

#[derive(Clone, Default)]
struct Edge<T>
    where T: Clone + Default
{
    from:  usize,
    to:    usize,
    inner: T,
}

//generate a matrix where row[i] =/= i,
//where columns only have unique elements
//and where there are no duplicate reverse edges ie (1,25) x (25,1)
//so this could be indicated as a triangle of the matrix FROM X TO where a 1 indicates the existance of an edge
//just take out the diagonal for no self edges.
//the number of possible edges then is n*(n-1) / 2
//edge 0 corresponds to 0x1, 1 to
///   1 2 3 4 5  
/// 1 x x x x x
/// 2 o x x x x    
/// 3 o o x x x    
/// 4 o o o x x
/// 5 o o o o x
/// so if i generate 10, how do i get the row and column?
/// so x = n(n-1)/2 , given x solve for n?
/// 2x = n(n-1) = n2 - n
/// 0 = n^2 - n - 2x
/// (1 + sqrt(1+8x)) / 2 (quadratic formula)
///  (1 + sqrt(81)) / 2 = 5
/// so ceil = the row
/// alternatively 2*
fn random_unique_nonself(n: usize, d: usize) {}

// impl<N,E,G> Graph<N,E,G>
// where N : Clone + Default ,
//       E : Clone + Default,
//       G : Clone
// {
//     fn new(nodes: Vec<Node<N>>, edges: Vec<Edge<E>>, inner: G) -> Self
//     {
//         Graph { nodes, edges, inner }
//     }

//     ///panics : d >= n
//     fn new_random_degree(n: usize, d: usize, inner : G) -> Self
//     {
//         let mut nodes = vec![];
//         let mut edges = vec![];
//         assert!(d < n , "degree must be lower than the size of the graph");
//         for i in 0..n
//         {
//             nodes.push(Node { inner : N::default() });
//         }
//         //ok, efficiently drawing n non-duplicate non-self edges :
//         //create array of n elements 1:n
//         //shuffle it, swap elements where n[i]==i with i+1%n
//         //

//         for i in 0..n
//         {
//             for o in 0..d
//             {
//                 let mut to = (random_range(1, n) + i) % n;
//                 edges.push(Edge { from : i, to, inner : E::default() });
//             }
//         }
//         let mut g = Graph { nodes, edges, bounds };
//         return g;
//     }

//     fn new_full(n: usize, bounds: Rect) -> Self
//     {
//         let nodes = vec![
//             Node { position: Vec2::ZERO,
//                    velocity: Vec2::ZERO,
//                    accel:    Vec2::ZERO, };
//             n
//         ];
//         let mut edges = vec![];
//         for i in 0..n
//         {
//             for j in 0..n
//             {
//                 if i == j
//                 {
//                     continue;
//                 }
//                 edges.push(Edge { from:  i,
//                                   to:    j,
//                                   force: 0.0, });
//             }
//         }
//         return Graph { nodes, edges, bounds };
//     }
// }
