use std::path::{PathBuf, Path};
use std::io::{self, Read, BufReader};
use std::fs::File;
use std::collections::HashMap;

use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::scanner::ScanError;

#[derive(Debug)]
pub struct Game {
    story: HashMap<String, Node>,
    current_node: String,
    state: State,
    assets: PathBuf,
    starting_node_name: Option<String>,
}

impl Game {
    pub fn new(assets: PathBuf) -> Self {
        Game {
            story: HashMap::new(),
            current_node: "".to_owned(),
            state: State::Load,
            assets: assets,
            starting_node_name: None,
        }
    }

    pub fn process(&mut self, in_text: &String, out_text: &mut String) {
        match self.state {
            State::Load => {
                let path = self.assets.join(in_text);
                if let Err(error) = self.open_story(path) {
                    println!("{:?}", error);
                } else {
                    if let Err(error) = self.print_choices(out_text) {
                        println!("{:?}", error);
                    }
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
        let mut s = in_text.clone();
        {
            let current_node = try!(self.story.get(&self.current_node).ok_or(Error::CurrentNodeInvalid));
            let temp_node = current_node.choices.get(in_text).ok_or(Error::ChoiceNotFound);
            self.current_node = try!(temp_node).0.clone();
            s.push('\n');
            s.push_str(try!(temp_node).1.as_str());
        }
        s.push('\n');
        s.push_str(try!(self.story.get(&self.current_node).ok_or(Error::CurrentNodeInvalid)).enter.as_str());
        s.push('\n');
        try!(self.print_choices(&mut s));
        out_text.push_str(s.as_str());
        Ok(())
    }

    fn print_choices(&mut self, out_text: &mut String) -> Result<(), Error> {
        out_text.push_str("Choices:\n");
        for choice in try!(self.story.get(&self.current_node).ok_or(Error::CurrentNodeInvalid)).choices.keys() {
            out_text.push_str(choice.as_str());
            out_text.push(' ');
        }
        out_text.push('\n');
        Ok(())
    }

    fn open_story<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let f = try!(File::open(path).map_err(|err| Error::Io(err)));
        let mut reader = BufReader::new(f);
        let mut buffer = String::new();
        let _ = try!(reader.read_to_string(&mut buffer).map_err(|err| Error::Io(err)));
        let docs = try!(YamlLoader::load_from_str(&buffer).map_err(|err| Error::Yaml(err)));
        self.story = HashMap::new();
        self.process_yaml(&docs[0]);
        self.state = State::Play;
        self.current_node = try!(self.starting_node_name.take().ok_or(Error::NoStartingNodeFound));
        Ok(())
    }

    fn process_yaml(&mut self, yaml: &Yaml) {
        match yaml {
            &Yaml::Hash(ref hashes) => {
                for hash in hashes {
                    match hash.0 {
                        &Yaml::String(ref key) => {
                            match key.as_str() {
                                "start" => {
                                    match hash.1 {
                                        &Yaml::String(ref value) => {
                                            self.starting_node_name = Some(value.clone());
                                        }
                                        _ => (),
                                    }
                                },
                                name => {
                                    let mut node = Node {
                                        enter: "".to_owned(),
                                        choices: HashMap::new(),
                                    };
                                    match hash.1 {
                                        &Yaml::Hash(ref hashes) => {
                                            for hash in hashes {
                                                match hash.0 {
                                                    &Yaml::String(ref entry_type) => {
                                                        match entry_type.as_str() {
                                                            "choices" => {
                                                                match hash.1 {
                                                                    &Yaml::Hash(ref hashes) => {
                                                                        for hash in hashes {
                                                                            match hash.0 {
                                                                                &Yaml::String(ref choice_name) => {
                                                                                    match hash.1 {
                                                                                        &Yaml::Hash(ref hashes) => {
                                                                                            for hash in hashes {
                                                                                                match hash.0 {
                                                                                                    &Yaml::String(ref choice_key) => {
                                                                                                        match choice_key.as_str() {
                                                                                                            "target" => {
                                                                                                                match hash.1 {
                                                                                                                    &Yaml::String(ref target_node_name) => {
                                                                                                                        if node.choices.get(choice_name).is_some() {
                                                                                                                            node.choices.get_mut(choice_name).unwrap().0 = target_node_name.clone();
                                                                                                                        } else {
                                                                                                                            node.choices.insert(choice_name.clone(), (target_node_name.clone(), "".to_owned()));
                                                                                                                        }
                                                                                                                    },
                                                                                                                    _ => (),
                                                                                                                }
                                                                                                            },
                                                                                                            "text" => {
                                                                                                                match hash.1 {
                                                                                                                    &Yaml::String(ref choice_taken_text) => {
                                                                                                                        if node.choices.get(choice_name).is_some() {
                                                                                                                            node.choices.get_mut(choice_name).unwrap().1 = choice_taken_text.clone();
                                                                                                                        } else {
                                                                                                                            node.choices.insert(choice_name.clone(), ("".to_owned(), choice_taken_text.clone()));
                                                                                                                        }
                                                                                                                    }
                                                                                                                    _ => (),
                                                                                                                }
                                                                                                            },
                                                                                                            _ => (),
                                                                                                        }
                                                                                                    },
                                                                                                    _ => (),
                                                                                                }
                                                                                            }
                                                                                        },
                                                                                        _ => (),
                                                                                    }
                                                                                },
                                                                                _ => (),
                                                                            }
                                                                        }
                                                                    }
                                                                    _ => (),
                                                                }
                                                            },
                                                            "enter" => {
                                                                match hash.1 {
                                                                    &Yaml::String(ref enter_text) => {
                                                                        node.enter = enter_text.clone();
                                                                    },
                                                                    _ => (),
                                                                }
                                                            },
                                                            _ => (),
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        }
                                        _ => (),
                                    }
                                    self.story.insert(name.to_string(), node);
                                },
                            }
                        }
                        _ => (),
                    }
                }
            },
            other => println!("{:?}", other),
        }
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
    Yaml(ScanError),
    ChoiceNotFound,
    CurrentNodeInvalid,
    NoStartingNodeFound,
}

#[derive(Debug, Eq, PartialEq)]
struct Node {
    enter: String,
    choices: HashMap<String, (String, String)>,
}
