use std::collections::HashSet;
use std::env;

use rand::seq::SliceRandom;
use rand::Rng;
//use std::fs::OpenOptions;
//use std::io::Write;

use itertools::Itertools;
use std::iter::FromIterator;
use std::fs::read_to_string;
use std::str;
use colored::Colorize;

use rayon::prelude::*;

fn subsets<T: Clone>(items: &Vec<T>) -> Vec<Vec<T>> {
    (0..items.len())
        .map(|count| items.clone().into_iter().combinations(count))
        .flatten()
        .collect()
}

fn are_neighboors(tubing1: &Vec<Vec<usize>>, tubing2: &Vec<Vec<usize>>) -> bool {
    let mut count = 0;
    for tube in tubing1.iter() {
        if !tubing2.contains(tube) {
            count += 1;
        }
    }
    return count == 1;
}

fn are_compatible(tube1: &Vec<usize>, tube2: &Vec<usize>, g: &Graph) -> bool {
    let t1: std::collections::HashSet<&usize> = HashSet::from_iter(tube1);
    let t2: std::collections::HashSet<&usize> = HashSet::from_iter(tube2);

    if t1 == t2 {return false;};

    if t1.is_subset(&t2) || t2.is_subset(&t1) {return true};
    if t1.is_disjoint(&t2) {
        for e in g.edges.iter() {
            if (t1.contains(&e[0]) && t2.contains(&e[1])) || (t1.contains(&e[1]) && t2.contains(&e[0])) {return false;};
        }
    } else {return false;};
    return true;
}

fn compatible_with(partial_tubing: &Vec<Vec<usize>>, g: &mut Graph) -> HashSet<Vec<Vec<usize>>> {
    let mut result = HashSet::new();
    let mut to_add;

    //println!("What is compatible with {}?", format!("{:?}", partial_tubing));

    if g.tubes == None {g.find_tubes();};

    for tube1 in g.tubes.as_ref().unwrap().iter() {
        to_add = true;
        for tube2 in partial_tubing.iter() {
            if !are_compatible(&tube1, &tube2, g) {to_add = false;};
        }
        if to_add {
            let mut temp = partial_tubing.clone();
            temp.push(tube1.clone());
            result.insert(temp);
        };
    }
    //println!("returning {}", format!("{:?}", result));
    return result;
}

fn new_tube<'a> (v1: &'a Vec<Vec<usize>>, v2: &'a Vec<Vec<usize>>) -> Option<&'a Vec<usize>> {
    for tube in v1.iter() {
        if !v2.contains(tube) {return Some(&tube);};
    }
    return None;
}

struct Fhp<'a> {
    end: usize,
    start: usize,
    path: Vec<&'a Vec<Vec<usize>>>,
    alr_seen: HashSet<&'a Vec<usize>>,
}

impl Fhp<'_> {
    fn show(&self) {
        for tubing in &self.path {
            println!("{:?}", tubing);
        }
    }
}

struct Flipgraph<'a>{
    g: &'a Graph,
    vertices: Vec<&'a Vec<Vec<usize>>>,
    neighboorlist: Vec<Vec<&'a Vec<Vec<usize>>>>,
}
impl Flipgraph<'_> {

    fn find_fhp_rand(&self, tries: usize) -> Option<Fhp> {
        let res: Result<Vec<usize>, Fhp> =
            (0..tries)
            .collect::<Vec<_>>()
			.par_iter()
			.map(|_|
				self.try_for_one_fhp())
			.collect();
        match res {
            Ok(_) => None,
            Err(path) => Some(path),
        }
    }

    fn try_for_one_fhp(&self) -> Result<usize, Fhp> {
        let mut flips = (0..self.g.vertices.len()-1).collect::<Vec<_>>();
        let mut flipped: bool;
        flipped = true;
        let start = rand::thread_rng().gen_range(0..self.vertices.len());
        let mut path = Fhp {
            end: start,
            start: start,
            path: Vec::new(),
            alr_seen: HashSet::new(),
        };
        for tube in self.vertices[start].iter() {
            path.alr_seen.insert(tube);
        }
        path.path.push(&self.vertices[path.start]);
        while flipped {
            if path.alr_seen.len() == self.g.tubes.as_ref().unwrap().len() {
                return Err(path);
            }
            flipped = false;
            flips.shuffle(&mut rand::thread_rng());
            for flip in flips.iter() {
                let newtube = new_tube(&self.neighboorlist[path.end][*flip], &self.vertices[path.end]).unwrap();
                if !path.alr_seen.contains(newtube) {
                    path.alr_seen.insert(newtube);

                    path.end = self.vertices.iter().position(|&r| r == self.neighboorlist[path.end][*flip]).unwrap();
                    path.path.push(self.vertices[path.end]);
                    flipped = true;
                    break;
                }
            }
        }

        Ok(0)
    }

    fn find_fhc_rand(&self, tries: usize) -> Option<Fhp> {
        let res: Result<Vec<usize>, Fhp> =
            (0..tries)
            .collect::<Vec<_>>()
			.par_iter()
			.map(|_|
				self.try_for_one_fhc())
			.collect();
        match res {
            Ok(_) => None,
            Err(path) => Some(path),
        }
    }

    fn try_for_one_fhc(&self) -> Result<usize, Fhp> {
        let mut flips = (0..self.g.vertices.len()-1).collect::<Vec<_>>();
        let mut flipped: bool;
        flipped = true;
        let start = rand::thread_rng().gen_range(0..self.vertices.len());
        let mut path = Fhp {
            end: start,
            start: start,
            path: Vec::new(),
            alr_seen: HashSet::new(),
        };
        path.path.push(&self.vertices[path.start]);
        while flipped {
            if path.alr_seen.len() == self.g.tubes.clone().unwrap().len() && path.start == path.end {
                return Err(path);
            }
            flipped = false;
            flips.shuffle(&mut rand::thread_rng());
            for flip in flips.iter() {
                let newtube = new_tube(&self.neighboorlist[path.end][*flip], &self.vertices[path.end]).unwrap();
                if !path.alr_seen.contains(newtube) {
                    path.alr_seen.insert(newtube);

                    path.end = self.vertices.iter().position(|&r| r == self.neighboorlist[path.end][*flip]).unwrap();
                    path.path.push(self.vertices[path.end]);
                    flipped = true;
                    break;
                }
            }
        }
        Ok(0)
    }
}
struct Graph {
    vertices: Vec<usize>,
    edges: Vec<[usize; 2]>,
    tubes: Option<HashSet<Vec<usize>>>,
    tubings: Option<HashSet<Vec<Vec<usize>>>>,
}

 impl Graph {
    fn is_connected(&self, vertices: &Vec<usize>) -> bool{
        if vertices.is_empty() {return false};
        if vertices.len() == 1 {return true};
        let mut active = Vec::new();
        let mut new = Vec::new();
        let mut found = Vec::new();
        active.push(vertices[0]);
        new.push(vertices[0]);

        loop{
            if active.is_empty() {break;};
            new.clear();

            for v in active.iter() {
                for e in self.edges.iter() {
                    if e[0] == *v && !found.contains(&e[1]) && vertices.contains(&e[1]) {found.push(e[1]); new.push(e[1])};
                    if e[1] == *v && !found.contains(&e[0]) && vertices.contains(&e[0]) {found.push(e[0]); new.push(e[0])};
                }
            }
            active = new.clone();
        }

        for v in vertices.iter() {
            if !found.contains(v) {return false;};
        }
        return true;
    }
    fn find_tubes(&mut self) {
        match self.tubes {
            Some(_) => {return;}
            None => {
                let mut tubes = HashSet::new();
                for subset in subsets(&self.vertices).iter() {
                    if self.is_connected(&subset) {tubes.insert(subset.to_vec());}
                }
                self.tubes = Some(tubes);
            }
        }
        return;
    }
    fn find_tubings(&mut self) {
        match self.tubings {
            Some(_) => return,
            None => {
                let mut tubings = HashSet::new();
                for v in self.vertices.iter() {
                    tubings.insert(vec![vec![*v]]);
                }
                let mut new = HashSet::new();
                let mut to_add;
                for _i in 2..self.vertices.len() {
                    for partial_tubing in tubings {
                        for new_partial_tubing in compatible_with(&partial_tubing, self).iter() {
                            to_add = new_partial_tubing.clone();
                            to_add.sort();
                            new.insert(to_add);
                        }
                    }
                    tubings = new.clone();
                    new.clear();
                }
                self.tubings = Some(tubings);
            }
        }
    }
}

fn flipgraph(g: &mut Graph) -> Flipgraph {
    g.find_tubings();
    let mut flipgraph = Flipgraph {
        g: g,
        vertices: Vec::from_iter(g.tubings.as_ref().unwrap()),
        neighboorlist: Vec::new(),
    };
    for _i in 0..g.tubings.as_ref().unwrap().len() {
        flipgraph.neighboorlist.push(Vec::new());
    }
    for (i,v) in g.tubings.as_ref().unwrap().iter().enumerate() {
        for w in g.tubings.as_ref().unwrap().iter() {
            if are_neighboors(v,w) {flipgraph.neighboorlist[i].push(w)};
        }
    }
    return flipgraph;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let source = &args[1];
    let paths_or_cycles = &args[2];
//    let n = args[3].parse::<usize>().unwrap();
    let tries = args[3].parse::<usize>().unwrap();
//    let save = &args[4];
    let talk = &args[4];

//    let mut file = OpenOptions::new().write(true).truncate(true).open(save).expect("failed to open file");

    let mut graphs: Vec<Graph> = Vec::new();

    for edgelist in read_to_string(source).expect("Read failed.").lines() {
        let edges_as_strings: Vec<&str> = edgelist[2..edgelist.len()-2].split("), (").collect();
        let mut edges: Vec<[usize; 2]> = Vec::new();
        for edge_str in edges_as_strings.iter() {
            let edge_vec: Vec<usize> = edge_str.split(", ").map(|r| r.parse::<usize>().unwrap()).collect();
            edges.push([edge_vec[0], edge_vec[1]]);
        }
        let mut vertices = Vec::new();
        for [a, b] in edges.iter() {
            if !vertices.contains(a) {
                vertices.push(*a);
            }
            if !vertices.contains(b) {
                vertices.push(*b);
            }
        }
        graphs.push(
            Graph {
            vertices: vertices,
            edges: edges,
            tubes: None,
            tubings: None,
        })
    }
    let k = graphs.len();
    let mut all_have_one = true;
    for (i,mut graph) in graphs.into_iter().enumerate() {
        if !graph.is_connected(&graph.vertices) {continue};
        println!("\n\nTrying graph: {}/{}", i+1, k);
        println!("{:?}", graph.edges);

        let fg = flipgraph(&mut graph);
        match paths_or_cycles.as_str() {
            "paths" | "p" => match fg.find_fhp_rand(tries) {
                Some(p) => {
                    println!("{}\n", "Found one!:".green());
                    if talk == "y" {
                        p.show();
                    }
                },
                None => { println!("{} in {} attempts.\n","None found".red(), tries); all_have_one = false; 
//                file.write(format!("{:?}\n",graph.edges).replace("[","(").replace("]",")").as_bytes()).expect("write failed");

                },
            },
            "cycles" | "c" => match fg.find_fhc_rand(tries) {
                Some(c) => {
                    println!("{}\n", "Found one!:".green());
                    if talk == "y" {
                        c.show();
                    }
                },
                None => { println!("{} in {} attempts.\n","None found".red(), tries); all_have_one = false;
//                file.write(format!("{:?}\n",graph.edges).replace("[","(").replace("]",")").as_bytes()).expect("write failed");
                },
            },
            _ => (),
        }
    }
    match all_have_one {
        true => println!("\n\n All these graphs have facet hamiltonian paths/cycles"),
        false => println!("\n\n Facet hamitlonian paths/cycles not found for all graphs"),
    }
}





















#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connectivity() {
        let g = Graph {
            vertices: vec![1,2,3,4,5,6],
            edges: vec![
                 [1,2],
                 [2,3],
                 [4,5],
                 [5,6],
                 [2,6],
            ],
            tubes: None,
            tubings: None,
        };

        assert!(g.is_connected(&g.vertices));

        let h = Graph {
            vertices: vec![1,2,3,4,5,6],
            edges: vec![
                 [1,2],
                 [2,3],
                 [5,6],
                 [2,6],
            ],
            tubes: None,
            tubings: None,
        };

        assert!(!h.is_connected(&h.vertices));
    }

    #[test]
    fn tubes() {
        let mut g = Graph {
            vertices: vec![1,2,3,4,5],
            edges: vec![
                 [1,2],
                 [2,3],
                 [3,4],
                 [4,5],
            ],
            tubes: None,
            tubings: None,
        };
        g.find_tubes();

        assert_eq!(g.tubes, Some(
            HashSet::from(
                [
                    vec![1], vec![2], vec![3], vec![4], vec![5],
                    vec![1,2], vec![2,3], vec![3,4], vec![4,5],
                    vec![1,2,3], vec![2,3,4], vec![3,4,5],
                    vec![1,2,3,4], vec![2,3,4,5]
                ]
                )
            ));


        let mut h = Graph {
            vertices: vec![1,2,3,5,6],
            edges: vec![
                 [1,2],
                 [2,3],
                 [5,6],
                 [2,6],
            ],
            tubes: None,
            tubings: None,
        };
        h.find_tubes();
        assert_eq!(h.tubes, Some(
            HashSet::from(
                [
                    vec![1], vec![2], vec![3], vec![5], vec![6],
                    vec![1,2], vec![2,3], vec![2,6], vec![5,6],
                    vec![1,2,3], vec![1,2,6], vec![2,3,6], vec![2,5,6],
                    vec![1,2,3,6], vec![1,2,5,6], vec![2,3,5,6]
                ]
                )
            ));
    }

    #[test]
    fn tubings() {
        let mut g = Graph {
            vertices: vec![1,2,3,4,5],
            edges: vec![
                 [1,2],
                 [2,3],
                 [3,4],
                 [4,5],
            ],
            tubes: None,
            tubings: None,
        };
        g.find_tubings();

        let mut h = Graph {
            vertices: vec![1,2,3,4,5,6],
            edges: vec![
                 [1,2],
                 [2,3],
                 [3,4],
                 [4,5],
                 [5,6],
            ],
            tubes: None,
            tubings: None,
        };
        h.find_tubings();

        let mut l = Graph {
            vertices: vec![1,2,3,4,5,6,7],
            edges: vec![
                 [1,2],
                 [2,3],
                 [3,4],
                 [4,5],
                 [5,6],
                 [6,7],
            ],
            tubes: None,
            tubings: None,
        };
        l.find_tubings();

        let mut k = Graph {
            vertices: vec![1,2,3,4,5,6],
            edges: vec![
                 [1,2],
                 [1,3],
                 [1,4],
                 [1,5],
                 [1,6],

                 [2,3],
                 [2,4],
                 [2,5],
                 [2,6],

                 [3,4],
                 [3,5],
                 [3,6],

                 [4,5],
                 [4,6],

                 [5,6],
            ],
            tubes: None,
            tubings: None,
        };
        k.find_tubings();

        assert_eq!([42,132,429, 720], [g.tubings.unwrap().len(), h.tubings.unwrap().len(), l.tubings.unwrap().len(), k.tubings.unwrap().len()]);
    }

    #[test]
    fn neighboorcheck () {
        let a = vec![vec![1], vec![1,2], vec![1,2,3], vec![1,2,3,4]];
        let b = vec![vec![1], vec![1,2], vec![4], vec![1,2,3,4]];
        let c = vec![vec![2], vec![1,2], vec![4], vec![1,2,3,4]];

        assert!(are_neighboors(&a, &b));
        assert!(are_neighboors(&b, &c));
        assert!(!are_neighboors(&a, &c));
    }

    #[test]
    fn fhp_finder () {
        let mut g = Graph {
            vertices: vec![1,2,3,4,5],
            edges: vec![
                 [1,2],
                 [2,3],
                 [3,4],
                 [4,5],
            ],
            tubes: None,
            tubings: None,
        };
        g.find_tubings();
        let fg = flipgraph(&g);
        fg.find_fhp_rand(1);
    }

    

}
