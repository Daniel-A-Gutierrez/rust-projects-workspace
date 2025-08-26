use std::{clone, f32, iter::from_fn};

// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com
//
// Example 1-10: Motion 101 Acceleration
use nannou::{geom::Scalar, prelude::*, };

const EDGE_LENGTH: f32 = 500f32;
const MAX_VELOCITY: f32 = 100f32;
const DRAG_COEF: f32 = 0.5f32;
const TIME_STEP: f32 = 0.004f32;
const PI2 : f32 = f32::consts::PI * 2.0;
const NODE_REPULSION : f32 = 1.0;
fn main()
{
    nannou::app(model).update(update).run();
}

struct Model
{
    graph: Graph,
}

struct Graph
{
    nodes:  Vec<Node>,
    edges:  Vec<Edge>,
    bounds: Rect,
}

#[derive(Clone)]
struct Node
{
    position: Vec2,
    velocity: Vec2,
    accel:    Vec2,
}

struct Edge
{
    from:  usize,
    to:    usize,
    force: f32,
}

struct Mover
{
    position:     Point2,
    velocity:     Vec2,
    acceleration: Vec2,
    top_speed:    f32,
}

impl Graph
{
    fn new(nodes: Vec<Node>, edges: Vec<Edge>, bounds: Rect) -> Self
    {
        Graph { nodes,
                edges,
                bounds }
    }

    fn new_random(n: usize, e: usize, bounds: Rect) -> Self
    {
        let mut nodes = vec![];
        let mut edges = vec![];
        assert!(n > 1, "graph must have at least 2 nodes");
        let nr = f32::sqrt(n as f32) as f32;
        for i in 0..n
        {
            nodes.push(Node { position: Vec2::ZERO,
                              velocity: Vec2::ZERO,
                              accel:    Vec2::ZERO, });
        }
        for i in 0..e
        {
            let from = random_range(0, n);
            let mut to = random_range(0, n);
            while to == from
            {
                to = random_range(0, n);
            }
            edges.push(Edge { from,
                              to,
                              force: 0.0 });
        }
        let mut g =  Graph { nodes,
                       edges,
                       bounds };
        return g;
    }

    fn new_full(n:usize, bounds:Rect)-> Self
    {
        let nodes = vec![Node{position : Vec2::ZERO, velocity : Vec2::ZERO, accel : Vec2::ZERO}; n];
        let mut edges = vec![];
        for i in 0..n 
        {
            for j in 0..n 
            {
                if i == j {continue}
                edges.push(Edge{from : i , to : j, force : 0.0 });
            }
        }
        return Graph{nodes,edges,bounds};
    }

    fn update(&mut self)
    {
        //add spring forces from edges
        for edge in self.edges.iter_mut()
        {
            let displacement = self.nodes[edge.from].position - self.nodes[edge.to].position; // -2, 0 
            let force = EDGE_LENGTH - displacement.length(); // 1 - 2 = -1 
            edge.force = force;
            let force = v2fmul(displacement.normalize(), force); // 2,0
            self.nodes[edge.from].accel += force;
            self.nodes[edge.to].accel -= force;
        }
        for i in 0..self.nodes.len()
        {
            
            let (before, after) = self.nodes.split_at_mut(i);
            let before = before.iter_mut();
            let mut after = after.iter_mut();
            let node = after.next().expect("nodes must have at least 1 node");
            for other in before.chain(after)
            {
                //node.accel += Vec2::splat(NODE_REPULSION/100000.0) / ((node.position - other.position) * (node.position - other.position))
            }
            //acceleration update
            node.accel -= v2fmul(node.velocity, DRAG_COEF);
            //velocity update
            node.velocity += v2fmul(node.accel, TIME_STEP);
            //position update
            node.position += v2fmul(node.velocity, TIME_STEP);

            node.position = Vec2::new(node.position.x
                                          .clamp(self.bounds.left(), self.bounds.right()),
                                      node.position.y
                                          .clamp(self.bounds.bottom(), self.bounds.top()));
            node.accel = Vec2::ZERO;
        }
    }

    fn position_grid(&mut self)
    {
        let nr = ( self.nodes.len() as f32 ).sqrt();
        for (i,node) in self.nodes.iter_mut().enumerate()
        {
            node.position = Vec2::new(self.bounds.left(), self.bounds.bottom()) + 
                             Vec2::new(     (self.bounds.w() / nr) * (i % nr as usize) as f32,
                                            (i / nr as usize) as f32 * self.bounds.h());
        }
    }

    fn position_circle(&mut self, radius : f32)
    {
        let n = self.nodes.len();
        for (i,node) in self.nodes.iter_mut().enumerate()
        {
            node.position = v2fmul(Vec2::new( f32::cos(PI2 * i as f32 / n as f32) , f32::sin(PI2 * i as f32 / n as f32) ), radius);
        }
    }

    fn display(&self, draw: &Draw)
    {
        // Display circle at x position
        // draw.ellipse()
        //     .xy(self.position)
        //     .w_h(4.0, 4.0) //.w_h(48.0, 48.0)
        //     .gray(0.5)
        //     .stroke(BLACK)
        //     .stroke_weight(2.0);
        for edge in self.edges.iter()
        {
            draw.line()
                .start(self.nodes[edge.from].position)
                .end(self.nodes[edge.to].position)
                .hsl(edge.force/EDGE_LENGTH/2.0, 1.0, 0.5);
        }
        for node in self.nodes.iter()
        {
            draw.ellipse().xy(node.position).w_h(5.0, 5.0).stroke(BLACK).stroke_weight(2.0).gray(0.0);
        }
    }
}

fn model(app: &App) -> Model
{
    let _window = app.new_window()
                     .size(1920, 1080)
                     .view(view)
                     .build()
                     .unwrap();
    let mut graph = Graph::new_full(24, app.window_rect());//new_random(10, 45, app.window_rect());
    graph.position_circle(EDGE_LENGTH/4.0);
    Model { graph }
}

fn update(app: &App, m: &mut Model, _update: Update)
{
    // update gets called just before view every frame
    m.graph.update();
    if app.keys.down.contains(&Key::R)
    {
        m.graph.position_circle(EDGE_LENGTH/4.0);
    }
}

fn view(app: &App, m: &Model, frame: Frame)
{
    // Begin drawing
    let draw = app.draw();

    draw.background().color(WHITE);

    if app.keys.down.contains(&Key::Space)
    {
        m.graph.nodes.iter().for_each(|n| println!("{}", n.position));
    }
    
    //println!("{:?}",app.window_rect());
    m.graph.display(&draw);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn v2fmul(a: Vec2, b: f32) -> Vec2
{
    return Vec2::new(a.x * b, a.y * b);
}
