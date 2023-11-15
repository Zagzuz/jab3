use crate::proto::InputFile;
use compact_str::CompactString;
use std::collections::HashMap;

pub type Files = HashMap<CompactString, InputFile>;

pub trait GetFiles {
    fn get_files(&self) -> Files;
    fn any_need_upload(&self) -> bool {
        self.get_files().values().any(|file| file.need_upload())
    }
}
