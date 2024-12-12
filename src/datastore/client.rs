use serde_json::to_vec_pretty;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub trait Savable {
    type A;

    fn add(client: &RefCell<Client>, item: &Self::A);
    fn check(client: &RefCell<Client>, item: &Self::A) -> bool;
    fn save(client: &RefCell<Client>) -> anyhow::Result<()>;
}

pub trait Restorable {
    fn restore(&mut self) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct Client {
    file: PathBuf,
    set: HashSet<String>,
}

impl Client {
    pub fn new(file: PathBuf) -> Client {
        Self {
            file,
            set: HashSet::new(),
        }
    }
}
pub struct RssSave {}

impl Restorable for Client {
    fn restore(&mut self) -> anyhow::Result<()> {
        if self.file.is_file() {
            let mut file = File::open(self.file.as_path())?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;

            let set: HashSet<String> = serde_json::from_str(&buffer)?;
            self.set = set;
            anyhow::Ok(())
        } else {
            let mut f = File::create_new(&self.file)?;
            f.write_all(&to_vec_pretty(&vec![""])?)?;
            anyhow::Ok(())
        }
    }
}
impl Savable for RssSave {
    type A = String;

    fn add(client: &RefCell<Client>, item: &Self::A) {
        client.borrow_mut().set.insert(item.to_owned());
        save::<RssSave>(client).expect("TODO: panic message");
    }

    fn check(client: &RefCell<Client>, item: &Self::A) -> bool {
        client.borrow().set.contains(item)
    }

    fn save(client: &RefCell<Client>) -> anyhow::Result<()> {
        let mut file = File::create(client.borrow().file.as_path())?;
        serde_json::to_string(&client.borrow().set.iter().collect::<Vec<_>>())?;
        file.write_all(&to_vec_pretty(
            &client.borrow().set.iter().collect::<Vec<_>>(),
        )?)?;
        anyhow::Ok(())
    }
}

fn save<Op: Savable>(client: &RefCell<Client>) -> anyhow::Result<()> {
    Op::save(client)
}
pub fn check<Op: Savable>(item: &Op::A, client: &RefCell<Client>) -> bool {
    if Op::check(client, item) {
        true
    } else {
        Op::add(client, item);
        false
    }
}
