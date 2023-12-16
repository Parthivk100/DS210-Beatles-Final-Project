use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Clone, Copy)]
pub struct Song {
    pub Energy: f32,
    pub Acoust: f32,
    pub Valence: f32,
    pub Name: &'static str,
    pub Year: u32,
}

pub fn read_songs_from_csv(file_path: &str) -> Result<Vec<Song>, Box<dyn Error>> {
    let mut songs: Vec<Song> = Vec::new();

    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().from_reader(file);
    for result in rdr.records() {
        let record = result?;
        if record.len() >= 5 {
            let name = record[0].to_string();
            let energy: f32 = record[1].parse()?;
            let valence: f32 = record[2].parse()?;
            let acoust: f32 = record[3].parse()?;
            let year: u32 = record[4].parse()?;
            let song = Song {
                Name: Box::leak(name.into_boxed_str()),
                Energy: energy,
                Valence: valence,
                Acoust: acoust,
                Year: year,
            };
            songs.push(song);
        }
    }

    Ok(songs)
}