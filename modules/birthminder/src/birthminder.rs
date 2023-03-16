use async_trait::async_trait;
use chrono::{Local, NaiveDate};
use compact_str::CompactString;
use std::{
    collections::{hash_map::Entry, HashMap},
    str::FromStr,
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

use eyre::{bail, eyre};
use log::debug;

use api::{
    basic_types::UserId,
    proto::{Message, User},
};
use bot::{bot::command::BotCommandInfo, communicator::Communicate, module::Module};

#[derive(Debug, Default)]
pub struct Birthminder {
    map: Arc<RwLock<BirthdayMap>>,
}

impl Birthminder {
    pub fn new() -> Self {
        Default::default()
    }

    // todo: config with different kinds of wishes

    pub fn save(&mut self, user: &User, date: NaiveDate) -> eyre::Result<()> {
        let mut data = self.map.write().expect("birthday map lock poisoned");
        match data.0.entry(date) {
            Entry::Occupied(mut o) => {
                o.get_mut().push(user.into());
            }
            Entry::Vacant(v) => {
                v.insert(vec![user.into()]);
            }
        };
        Ok(())
    }

    pub fn next_birthdays(&self) -> (NaiveDate, Vec<&UserData>) {
        // let map = self.map.read().expect("birthday map lock poisoned");
        todo!()
    }

    pub fn greet_thread(&mut self) -> JoinHandle<()> {
        /*let mut scheduler = Scheduler::new();
        let map = self.map.clone();
        thread::spawn(move || {
            scheduler.every(1.day()).at("12:00 pm").run(move || {
                let mut map = map.read().expect("birthday map lock poisoned");
                let Some(birthdays) = map.today_birthdays() else {
                    return;
                };
                for user_data in birthdays {

                }
            });
        })*/
        todo!()
    }
}

#[derive(Debug, Default)]
struct BirthdayMap(HashMap<NaiveDate, Vec<UserData>>);

impl BirthdayMap {
    pub fn _today_birthdays(&self) -> Option<&Vec<UserData>> {
        let today = Local::now().date_naive();
        let Some(list) = self.0.get(&today) else {
            return None;
        };
        Some(list)
    }

    pub fn _birthday_list(&self) -> Vec<(NaiveDate, UserData)> {
        self.0
            .iter()
            .flat_map(|(date, users)| users.iter().map(|user| (*date, user.clone())))
            .collect()
    }

    pub fn _next_birthdays(&self) -> (NaiveDate, Vec<&UserData>) {
        // let mut today = Utc::now().date_naive();
        // self.0.iter().min_by_key(|(date, _)| date.);
        todo!()
    }
}

#[cfg_attr(test, derive(Default, Eq, PartialEq))]
#[derive(Debug, Clone)]
pub struct UserData {
    first_name: CompactString,
    last_name: Option<CompactString>,
    username: Option<CompactString>,
    id: UserId,
}

impl From<&User> for UserData {
    fn from(user: &User) -> Self {
        UserData {
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            username: user.username.clone(),
            id: user.id,
        }
    }
}

enum CommandName {
    Set,
    Next,
}

impl FromStr for CommandName {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "set" => Ok(CommandName::Set),
            "next" => Ok(CommandName::Next),
            _ => bail!("failed to recognize '{s}' as a possible command"),
        }
    }
}

#[async_trait]
impl Module for Birthminder {
    async fn try_execute_command(
        &mut self,
        _comm: &dyn Communicate,
        cmd: &BotCommandInfo,
        message: &Message,
    ) -> eyre::Result<()> {
        let name = match CommandName::from_str(cmd.name()) {
            Ok(name) => name,
            Err(err) => {
                debug!("{err}");
                return Ok(());
            }
        };
        match name {
            CommandName::Set => {
                let user = message.from.as_ref().ok_or(eyre!(
                    "no user info to save birthday, message = {message:?}"
                ))?;
                let date = NaiveDate::parse_from_str(cmd.query().as_str(), "%d.%m")?;
                self.save(user, date)?;
            }
            CommandName::Next => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;
    use compact_str::ToCompactString;

    fn user_with_id(id: UserId) -> User {
        User {
            id,
            is_bot: false,
            first_name: id.to_compact_string(),
            last_name: None,
            username: None,
            language_code: None,
            is_premium: None,
            added_to_attachment_menu: None,
            can_join_groups: None,
            can_read_all_group_messages: None,
            supports_inline_queries: None,
        }
    }

    #[test]
    fn get_birthday_list() {
        let mut b = Birthminder::new();

        let user1 = user_with_id(0);
        let date1 = NaiveDate::parse_from_str("2001-01-01", "%Y-%m-%d").unwrap();
        b.save(&user1, date1.clone()).unwrap();

        let user21 = user_with_id(1);
        let date2 = NaiveDate::parse_from_str("2002-02-02", "%Y-%m-%d").unwrap();
        b.save(&user21, date2.clone()).unwrap();

        let user22 = user_with_id(2);
        b.save(&user22, date2.clone()).unwrap();

        let user3 = user_with_id(3);
        let date3 = NaiveDate::parse_from_str("2003-09-05", "%Y-%m-%d").unwrap();
        b.save(&user3, date3.clone()).unwrap();

        assert_eq!(
            b.list().unwrap(),
            vec![
                (date1, UserData::from(user1)),
                (date2.clone(), UserData::from(user21)),
                (date2, UserData::from(user22)),
                (date3, UserData::from(user3)),
            ]
        );
    }
}
