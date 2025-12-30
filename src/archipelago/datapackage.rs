use std::{fs::File, io::Write};

use serde::{Deserialize, Serialize};

use crate::archipelago::MyDataPackage;

use super::server_messages::DataPackageObject;

#[derive(Deserialize, Serialize)]
pub(super) struct DataPackageSave {
    pub(super) game: String,
    pub(super) datapackage: MyDataPackage,
}

pub(super) fn save_datapackage(game: &str, datapackage: &MyDataPackage) -> Result<(), ()> {
    let Ok(datapackage_str) = serde_json::to_string(&DataPackageSave {
        game: game.to_string(),
        datapackage: datapackage.clone(),
    }) else {
        return Err(());
    };

    let Some(dir) = dirs::cache_dir() else {
        return Err(());
    };

    let dir_path = dir.join("elementipelago").join("datapackages");

    let file_path = dir_path.join(format!("{}.json", game));

    std::fs::DirBuilder::new()
        .recursive(true)
        .create(dir_path)
        .expect("could not create datapackage cache dir");

    match File::create(&file_path) {
        Ok(mut file) => file
            .write_all(datapackage_str.as_bytes())
            .expect("can not write to the file"),
        Err(e) => {
            eprintln!(
                "could not create file at {}, due to error: {:?}",
                file_path.to_str().unwrap(),
                e
            );
            return Err(());
        }
    };

    Ok(())
}
