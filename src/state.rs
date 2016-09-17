use std::path::Path;
use std::io::{self, BufReader, BufRead};
use std::fs::File;

use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct Game {
    story: Vec<Node>,
    current_node: usize,
    state: State,
}

impl Game {
    pub fn new() -> Self {
        Game {
            story: vec!(),
            current_node: 0,
            state: State::Load,
        }
    }

    pub fn process(&mut self, in_text: &String, out_text: &mut String) {
        match self.state {
            State::Load => {
                if let Err(error) = open_story(in_text).map_err(|err| Error::Io(err)) {
                    println!("{:?}", error);
                }
            },
            State::Play => {
                if let Err(error) = self.process_choice(in_text, out_text) {
                    println!("{:?}", error);
                }
            },
        }
    }

    fn process_choice(&mut self, in_text: &String, out_text: &mut String) -> Result<(), Error> {
        self.current_node = try!(try!(self.story.get(self.current_node).ok_or(Error::CurrentNodeInvalid)).choices.binary_search_by(|probe| probe.0.cmp(in_text)).map_err(|_| Error::ChoiceNotFound));
        let mut s = try!(self.story.get(self.current_node).ok_or(Error::CurrentNodeInvalid)).name.clone();
        s.push('\n');
        out_text.push_str(s.as_str());
        Ok(())
    }
}

#[derive(Debug)]
enum State {
    Load,
    Play,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    StateError(&'static str),
    ChoiceNotFound,
    CurrentNodeInvalid,
}

#[derive(Debug, Ord, Eq, PartialOrd, PartialEq)]
struct Node {
    name: String,
    choices: Vec<(String, usize)>,
}

fn open_story<P: AsRef<Path>>(p: P) -> io::Result<Node> {
    let f = try!(File::open(p));
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();
    let _ = try!(reader.read_line(&mut buffer));
    let docs = YamlLoader::load_from_str(&buffer).unwrap();
    let mut start_node_opt = None;
    start_node_opt = Some(process_yaml(&docs[0]));
    Ok(
        start_node_opt.unwrap()
    )
}

fn process_yaml(yaml: &Yaml) -> Node {
    match yaml {
        &Yaml::Array(ref v) => {
            for yaml in v {
                process_yaml(yaml);
            }
        },
        &Yaml::String(ref s) => {
            println!("{:?}", s);
        },
        _ => (),
    }
    Node {
        name: "".to_owned(),
        choices: vec!(),
    }
}
