use std::collections::HashMap;
use petgraph::prelude::*;
use rand::Rng;
mod csv_reader;
use csv_reader::{Song, read_songs_from_csv};

fn main() {
    
    let file_path = "/Users/parthivkrishnan/Downloads/The_Beatles_Cleaned_SEVAY.csv";
    match read_songs_from_csv(file_path) {
        Ok(songs) => {
            let graph = create_dgraph(songs.clone());
            let num_clusters = 3;
            let max_iterations = 100;
            let clusters = kmeans_cluster(&graph, num_clusters, max_iterations);
            let unique_clusters = get_unique_clusters(clusters, &graph);

            //print_clusters(unique_clusters, &graph);
            average_cluster_year(unique_clusters, &graph);
        }

        Err(err) => eprintln!("Error: {}", err),
    }


   
}

impl Song {
    fn similarity(&self, other: &Song) -> f32 {
        ((self.Energy - other.Energy).powi(2) 
        + (self.Acoust - other.Acoust).powi(2) 
        + (self.Valence - other.Valence).powi(2)).sqrt()
    }
}

fn add_edge(graph: &mut Graph<Song, f32, Directed>, song1: Song, song2: Song) {
    let node1 = graph.add_node(song1);
    let node2 = graph.add_node(song2);
    let similarity = graph[node1].similarity(&graph[node2]);
    graph.add_edge(node1, node2, similarity);
}

fn create_dgraph(songs: Vec<Song>) -> Graph<Song, f32, Directed> {
    let mut graph = Graph::<Song, f32, Directed>::new();

    for i in 0..songs.len() {
        let song1 = songs[i];

        for j in 0..songs.len() {
            let song2 = songs[j];

            if i != j {
                let sim = song1.similarity(&song2);
                if sim > 0.75 {
                    add_edge(&mut graph, song1, song2);
                }
            }
        }
    }

    graph
}

 
fn kmeans_cluster(graph: &Graph<Song, f32, Directed>, num_clusters: usize, max_iterations: usize) -> HashMap<usize, Vec<NodeIndex>> {
    let node_indices: Vec<NodeIndex> = graph.node_indices().collect();
    let num_nodes = node_indices.len();
    let feature_dimensions = 3; // Energy, Acoust, Valence

    // Initialize centroids randomly
    let mut centroids: Vec<Vec<f32>> = Vec::new();
    let mut rng = rand::thread_rng();
    for _ in 0..num_clusters {
        let random_index = rng.gen_range(0..num_nodes);
        let song = &graph[node_indices[random_index]];
        centroids.push(vec![song.Energy, song.Acoust, song.Valence]);
    }
    let mut new_clusters: HashMap<usize, Vec<NodeIndex>>;

    let mut clusters: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
    // Iterate until convergence or max_iterations
    for _iteration in 0..max_iterations {
        // Assign nodes to clusters
        clusters.clear();
        new_clusters = HashMap::new();
        for &node_index in &node_indices {
            let song = &graph[node_index];
            let mut min_distance = f32::INFINITY;
            let mut assigned_cluster = 0;

            for (cluster_index, centroid) in centroids.iter().enumerate() {
                let distance = song.similarity(&Song {
                    Energy: centroid[0],
                    Acoust: centroid[1],
                    Valence: centroid[2],
                    Name: "",
                    Year: 0,
                });

                if distance < min_distance {
                    min_distance = distance;
                    assigned_cluster = cluster_index;
                }
            }

            clusters.entry(assigned_cluster).or_insert_with(Vec::new).push(node_index);
        }

        // Update centroids
        for (cluster_index, indices) in &clusters {
            let centroid = indices
                .iter()
                .map(|&index| {
                    let song = &graph[index];
                    vec![song.Energy, song.Acoust, song.Valence]
                })
                .fold(vec![0.0; feature_dimensions], |acc, x| {
                    acc.iter().zip(x.iter()).map(|(a, b)| a + b).collect()
                })
                .iter()
                .map(|&sum| sum / indices.len() as f32)
                .collect();

            centroids[*cluster_index] = centroid;
        }

        clusters.extend(new_clusters);
    }

    clusters
}

fn get_unique_clusters(clusters: HashMap<usize, Vec<NodeIndex>>, graph: &Graph<Song, f32, Directed>) -> HashMap<usize, Vec<NodeIndex>> {
    let mut unique_clusters: HashMap<usize, Vec<NodeIndex>> = HashMap::new();

    for (cluster_index, indices) in clusters {
        let mut unique_indices: Vec<NodeIndex> = Vec::new();

        for &node_index in &indices {
            let song = &graph[node_index];
            let song_clone = Song {
                Energy: song.Energy,
                Acoust: song.Acoust,
                Valence: song.Valence,
                Name: song.Name,
                Year: song.Year,
            };

            if !unique_indices.iter().any(|&i| graph[i].similarity(song) < f32::EPSILON) {
                unique_indices.push(node_index);
            }
        }

        unique_clusters.insert(cluster_index, unique_indices);
    }

    unique_clusters
}


fn print_clusters(clusters: HashMap<usize, Vec<NodeIndex>>, graph: &Graph<Song, f32, Directed>) {
    for (cluster_index, indices) in clusters {
        println!("Cluster {}: ", cluster_index + 1);
        for &node_index in &indices {
            let song = &graph[node_index];
            println!(" - {}, {}", song.Name, song.Year);
        }
        println!();
    }
}

fn average_cluster_year(clusters: HashMap<usize, Vec<NodeIndex>>, graph: &Graph<Song, f32, Directed>) -> Vec<f64> {
    let mut years = Vec::new();
    for (cluster_index, indices) in clusters {
        println!("Cluster {}: ", cluster_index + 1);
        let mut aggregate = 0;
        for &node_index in &indices {
            let song = &graph[node_index];
            aggregate += song.Year;
        }
        println!("Average Year: {}", (aggregate as f64 / indices.len() as f64));
        years.push((aggregate as f64 / indices.len() as f64));
    }
    years
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test1() {
        let file_path = "/Users/parthivkrishnan/Downloads/The_Beatles_Cleaned_SEVAY.csv";
        match read_songs_from_csv(file_path) {
            Ok(songs) => {
                let graph = create_dgraph(songs.clone());
                let num_clusters = 3;
                let max_iterations = 100;
                let clusters = kmeans_cluster(&graph, num_clusters, max_iterations);
                let unique_clusters = get_unique_clusters(clusters, &graph);
                let temp = average_cluster_year(unique_clusters, &graph);

                for i in temp {
                    assert!(i >= 1963.0)
                }
            }

            Err(err) => eprintln!("Error: {}", err),
        }
    }

}


/*
This is my final project. This project aimed to uncover previously unseen trends in The Beatles and their music. The goal of this project
was to understand whether or not music from a given era of the Beatles was more similar to the music from that era, or if it's more similar
to music from a different era of the Beatles. The Beatles went through three big phases, which can generally be classified as their Early, 
Middle, and Late eras. People say that music from these different eras are very distinct and I wanted to do a project on that topic to see
whether or not that statement has any form of validity to it. To do this I began with creating a simple struct called Song, which represents
just that a song, with the Name, year, Energy level %, Acousticness %, and the Valence %. These three characteristics are generally pretty 
good at capturing the most important information about a song, and serve as a decent measure of how similar songs are. I then made a simple 
function to calculate the similarity between songs and I began the construction of the directed graph. To do this, I first made the add_edge
function which takes two songs turns then into nodes, calculates the similarity between them and uses that value as a weight. Using this as a helper function
I then made the create directed graph function, which does just that. It will only makes songs point to other songs if the similarity metric
is greater than 0.75. Also to represent directed graphs, rather than using Hashmaps which could be done, I decided to use the petgraph library
because of the fact that the implimentation using this was far simpler than it would have been doing it from scratch, and the documentation
of this library was also good which made working with it a good choice for this project. Now that I have a complete directed graph, I wanted to cluster the nodes that were most similar to each other, and to 
do this I used kmeans clustering. I implemented the logic of the algorithm as best I could. Next I created a unique clusters function, since
the clusters being returned by my kmeans function had some repeating values. To get around this, I simply made a function which took the output
of the kmeans function and returned the same type only with no repeating values.In addition to this, I created a print cluster function and a average year
function, to see what my code was doing. The print cluster function allows you to see the name and year of all the songs in each respective cluster
and the average year function finds the average year of each cluster. Moreover, I created a module called csv_reader which is where my struct
is defined and is also where my csv reading function is held. From the results that I have seen, it's pretty fair to say that generally that
Beatles music from a given era is more similar to music from that same era. This is because of the fact that the average year of each cluster
tend to be pretty distinct, with only value typically around 1965, another around 1967, and the last one being around 1968. These match up with
Early, Middle, and Late eras of the Beatles pretty closely. Thus we can affirm that what people have been saying about the Beatles music 
is in fact true.
*/







